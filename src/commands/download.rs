//! Main file handling 'download' command

use anyhow::{bail, Context, Result};
use log::{debug, info};
use std::fs::{copy, create_dir_all, File};
use std::{
    io::{self, Read},
    path::{Component, Path, PathBuf},
    time::Duration,
};
use tempfile::NamedTempFile;

/// Maximum permitted release asset download size (512 MiB).
const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;
/// Maximum time allowed for a release asset download.
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(10 * 60);

/// Finalizes a downloaded file, copying through private temporary storage across filesystems.
fn finalize_download(temporary_file: NamedTempFile, target: &Path) -> Result<File> {
    let file = match temporary_file.persist(target) {
        Ok(file) => Ok(file),
        Err(error) if error.error.raw_os_error() == Some(libc::EXDEV) => {
            // Keep staging outside the destination directory, which may be writable by others.
            let staged_file = NamedTempFile::new()
                .context("Cannot create private temporary file for cross-filesystem copy")?;
            copy(error.file.path(), staged_file.path())
                .context("Cannot stage downloaded file for cross-filesystem copy")?;
            copy(staged_file.path(), target)?;
            File::open(target)
        }
        Err(error) => Err(error.error),
    };

    file.with_context(|| format!("Cannot finalize downloaded file {}", target.display()))
}

/// Resolves an asset filename beneath `directory`, rejecting paths and separators.
fn safe_asset_path(directory: &Path, filename: &str) -> Result<PathBuf> {
    let path = Path::new(filename);
    let mut components = path.components();
    if filename.contains('\\')
        || !matches!(components.next(), Some(Component::Normal(_)))
        || components.next().is_some()
    {
        bail!("Unsafe release asset filename: {filename}");
    }
    Ok(directory.join(path))
}

/// Download a single release asset to a local directory.
///
/// Fetches `download_url` and writes the response body to `download_to/filename`.
/// The destination directory is created if it does not already exist.
/// Returns the full path of the saved file on success.
pub fn download_asset(
    filename: &String,
    download_url: &String,
    download_to: &PathBuf,
) -> Result<PathBuf> {
    let target_file_path = safe_asset_path(download_to, filename)?;
    info!("Downloading {} from {}", filename, download_url);

    let client = reqwest::blocking::Client::builder()
        .timeout(DOWNLOAD_TIMEOUT)
        .build()
        .context("Cannot configure download client")?;
    let response = client
        .get(download_url)
        .send()
        .with_context(|| format!("Cannot initiate download from {}", download_url))?;

    let status = response.status(); // for borrowing
    if status.is_success() {
        let expected_size = response.content_length();
        if expected_size.is_some_and(|size| size > MAX_DOWNLOAD_BYTES) {
            bail!("Release asset exceeds the maximum download size of {MAX_DOWNLOAD_BYTES} bytes");
        }
        // Ensure the directory exists
        create_dir_all(download_to)
            .with_context(|| format!("Cannot create directory {}", download_to.display()))?;
        // Keep the in-progress file in the private system temporary directory. A
        // user-selected destination may be group writable, so its entries cannot
        // protect an in-progress download from replacement by another user.
        let mut temporary_file =
            NamedTempFile::new().context("Cannot create private temporary file")?;

        debug!(
            "Saving download to temporary file: {}",
            temporary_file.path().display()
        );

        // Copy the response body to the file
        let download_result = (|| -> Result<()> {
            let mut limited = response.take(MAX_DOWNLOAD_BYTES + 1);
            let copied = io::copy(&mut limited, temporary_file.as_file_mut())
                .context("Cannot write downloaded data to file")?;
            if copied > MAX_DOWNLOAD_BYTES {
                bail!(
                    "Release asset exceeds the maximum download size of {MAX_DOWNLOAD_BYTES} bytes"
                );
            }
            if expected_size.is_some_and(|expected| expected != copied) {
                bail!("Downloaded asset size does not match the expected size");
            }
            Ok(())
        })();
        download_result?;
        let finalized_file = finalize_download(temporary_file, &target_file_path)?;
        #[cfg(unix)]
        finalized_file
            .set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o644))
            .context("Cannot set downloaded file permissions")?;

        info!("Download complete.\n");
        Ok(target_file_path)
    } else {
        // we use anyhow::bail! for errors originating here
        let mut error_body = String::new();
        response
            .take(8 * 1024)
            .read_to_string(&mut error_body)
            .unwrap_or_default();
        anyhow::bail!(
            // with bail! macro we early return with error
            "Download failed! Status: {}. URL: {}. Server response: {}",
            status,
            download_url,
            error_body
        );
        // the error will be propagated and logged at a higher level in main.rs
        // also, we have context added via `?` to trace the origin :)
    }
}

#[cfg(test)]
mod tests;
