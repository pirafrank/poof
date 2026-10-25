//! Main file handling 'download' command

use anyhow::{bail, Context, Result};
use log::{debug, info};
use std::{
    fs::OpenOptions,
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

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

    let response = reqwest::blocking::get(download_url)
        .with_context(|| format!("Cannot initiate download from {}", download_url))?;

    let status = response.status(); // for borrowing
    if status.is_success() {
        let expected_size = response.content_length();
        if expected_size.is_some_and(|size| size > MAX_DOWNLOAD_BYTES) {
            bail!("Release asset exceeds the maximum download size of {MAX_DOWNLOAD_BYTES} bytes");
        }
        // Ensure the directory exists
        std::fs::create_dir_all(download_to)
            .with_context(|| format!("Cannot create directory {}", download_to.display()))?;

        // Create the file path and open it for writing
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target_file_path)
            .with_context(|| format!("Cannot create file {}", target_file_path.display()))?;

        debug!("Saving to: {}", target_file_path.display());

        // Copy the response body to the file
        let download_result = (|| -> Result<()> {
            let mut limited = response.take(MAX_DOWNLOAD_BYTES + 1);
            let copied = io::copy(&mut limited, &mut file)
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
        if let Err(error) = download_result {
            std::fs::remove_file(&target_file_path).ok();
            return Err(error);
        }

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
