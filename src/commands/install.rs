//! Main file handling 'install' command

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    commands::{self, download::download_binary},
    core::selector::is_env_compatible,
    files::{
        archives, datadirs, filesys, magic::is_exec_by_magic_number,
        utils::get_stem_name_trimmed_at_first_separator,
    },
    github::client::{get_asset, get_release},
    utils::semver::SemverStringPrefix,
};
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info, warn};

pub fn install(repo: &str, tag: Option<&str>) -> Result<()> {
    process_install(repo, tag, false)
}

pub fn install_new_default(repo: &str, tag: Option<&str>) -> Result<()> {
    process_install(repo, tag, true)
}

fn process_install(repo: &str, tag: Option<&str>, is_update: bool) -> Result<()> {
    let cache_dir: PathBuf =
        datadirs::get_cache_dir().context("Failed to determine cache directory")?;
    debug!("Cache directory: {}", cache_dir.display());

    // download binary
    let release = get_release(repo, tag)
        .with_context(|| format!("Failed to get release information for {}", repo))?;
    let binary = get_asset(&release, is_env_compatible).with_context(|| {
        format!(
            "Failed to find compatible asset for release {}",
            release.tag_name()
        )
    })?;
    let version: String = release.tag_name().strip_v();
    let download_to = datadirs::get_binary_nest(&cache_dir, repo, &version);

    // prepare install dir, skip if already installed
    let install_dir = match prepare_install_dir(repo, &version)
        .with_context(|| format!("Failed to prepare install directory for {}", repo))?
    {
        Some(dir) => dir,
        None => {
            info!(
                "Skipping installation as version {} for {} seems already installed.",
                version, repo
            );
            return Ok(());
        }
    };

    // if not installed, download release asset
    download_binary(binary.name(), binary.browser_download_url(), &download_to)
        .with_context(|| format!("Failed to download binary {} version {}", repo, version))?;
    let downloaded_file = download_to.join(binary.name());

    // check if downloaded binary is an archive or an executable
    // and proceed accordingly.
    if is_exec_by_magic_number(&downloaded_file) {
        debug!("Downloaded file {} is an executable binary.", binary.name());
        let file_name = &downloaded_file
            .file_name()
            .ok_or_else(|| anyhow!("Failed to get filename from {}", downloaded_file.display()))?;
        // Get the stem name trimmed at the first separator for non-archived executable files.
        // This is useful to avoid installing files with names like "mytool-1.0.0" or "mytool-linux-x86_64"
        // and instead use just "mytool", which is how the binary will be used when in PATH.
        let exec_name = get_stem_name_trimmed_at_first_separator(file_name);
        install_binary(&downloaded_file, &install_dir, &exec_name, is_update)
            .with_context(|| format!("Failed to install executable {}", binary.name()))?;
    } else {
        // extract binary
        archives::extract_to_dir(&downloaded_file, &download_to)
            .with_context(|| format!("Failed to extract archive to {}", download_to.display()))?;
        debug!("Extracted to: {}", download_to.display());

        // install binary
        install_binaries(&downloaded_file, &install_dir, is_update).with_context(|| {
            format!(
                "Failed to install binaries for {} version {}",
                repo, version
            )
        })?;
    }

    info!("{} {} installed successfully.", repo, &version);
    commands::check::check_if_bin_in_path()?;
    Ok(())
}

// Result<Option<PathBuf>>: Some(install_dir) = proceed, None = skip/already done
// in this way we eliminate the std::process::exit(0)
fn prepare_install_dir(repo: &str, version: &str) -> Result<Option<PathBuf>> {
    let data_dir: PathBuf =
        datadirs::get_data_dir().context("Failed to determine data directory.")?;
    debug!("Data directory: {}", data_dir.display());
    let install_dir: PathBuf = datadirs::get_binary_nest(&data_dir, repo, version);

    debug!("Installing to: {}", install_dir.display());
    // Create the installation directory if it doesn't exist
    if !install_dir.exists() {
        // directory does not exist, create it
        std::fs::create_dir_all(&install_dir).with_context(|| {
            format!(
                "Failed to create installation directory {}",
                install_dir.display()
            )
        })?;
        debug!("Created install directory: {}", install_dir.display());
        Ok(Some(install_dir))
    } else {
        // path exists, check if it's a directory
        if install_dir.is_dir() {
            // it's a directory, check if it's empty
            let is_empty = install_dir
                .read_dir()
                .with_context(|| format!("Failed to read directory {}", install_dir.display()))?
                .next()
                .is_none();

            if is_empty {
                // directory exists but is empty, proceed (overwrite)
                debug!("Install directory exists but is empty, proceeding.");
                Ok(Some(install_dir)) // installation should proceed
            } else {
                // directory exists and is not empty: we assume it's already installed
                warn!(
                    "Version already installed. Check content in {}. Skipping installation.",
                    install_dir.display()
                );
                // we return None to indicate skipping
                Ok(None)
            }
        } else {
            // exists but is not a directory (like a file)
            // we interpret it as an error
            bail!(
                "Install path {} exists but is not a directory. Please remove it manually.",
                install_dir.display()
            );
        }
    }
}

fn install_binaries(archive_path: &Path, install_dir: &Path, is_update: bool) -> Result<()> {
    // TODO: ensure filesys::find_exec_files_from_extracted_archive returns Result if needed
    // assuming for now it returns Vec<PathBuf> and handles its own errors internally or doesn't fail often
    let execs_to_install: Vec<PathBuf> =
        filesys::find_exec_files_from_extracted_archive(archive_path);

    if execs_to_install.is_empty() {
        // we interpret this as an error
        bail!("No executables found to install. Please check the archive contents.");
    }

    for exec in execs_to_install {
        // if we have multiple executables, we install each one.
        // we assume that to have multiple executables, those were in an archive.
        let exec_name = exec
            .file_name()
            .ok_or_else(|| anyhow!("Failed to get filename from {}", exec.display()))?;
        install_binary(&exec, install_dir, &OsString::from(exec_name), is_update)
            .with_context(|| format!("Failed to install executable {}", exec.display()))?;
    }
    Ok(())
}

fn install_binary(
    exec: &PathBuf,
    install_dir: &Path,
    exec_stem: &OsString,
    is_update: bool,
) -> Result<()> {
    let installed_exec = install_dir.join(exec_stem);

    // copy the executable files to the install directory
    // Copy the binary to the install directory
    filesys::copy_file(exec, &installed_exec).with_context(|| {
        format!(
            "Failed to copy {} to install dir ({})",
            exec.display(),
            installed_exec.display()
        )
    })?;

    let bin_dir: PathBuf = datadirs::get_bin_dir().context("Failed to locate bin directory")?;

    // make them executable
    // Set executable permissions, platform-specific
    // Note: Windows does not require setting executable permissions
    #[cfg(not(target_os = "windows"))]
    {
        // Make the file executable on Unix-like systems
        filesys::make_executable(&installed_exec)
            .with_context(|| format!("Failed to make {} executable", installed_exec.display()))?;
        // Create a symlink in the bin directory, overwriting existing if the install is an update
        let symlink_path = bin_dir.join(exec_stem);
        filesys::create_symlink(&installed_exec, &symlink_path, is_update).with_context(|| {
            format!("Failed to create symlink for {}", installed_exec.display())
        })?;
    }
    Ok(())
}
