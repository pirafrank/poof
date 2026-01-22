//! Main file handling 'install' command

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    commands::{self, download::download_asset},
    files::{
        archives, datadirs, filesys, magic::is_exec_by_magic_number,
        utils::get_stem_name_trimmed_at_first_separator,
    },
    github::{
        client::{get_assets, get_release},
        models::{Release, ReleaseAsset},
    },
    utils::semver::SemverStringPrefix,
};
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info, warn};

pub fn install(repo: &str, tag: Option<&str>) -> Result<()> {
    let (release, assets) = select_assets(repo, tag)?;
    let version: String = release.tag_name().strip_v();

    let install_dir = get_install_dir(repo, &version)?;
    if check_if_installed(&install_dir)? {
        info!(
            "Skipping installation as version {} for {} seems already installed.",
            version, repo
        );
        return Ok(());
    } else {
        // installation should proceed, prepare install directory
        prepare_install_dir(&install_dir)?;
    }

    // get cache directory as temporary download directory
    let cache_dir: PathBuf =
        datadirs::get_cache_dir().context("Failed to determine cache directory")?;
    debug!("Cache directory: {}", cache_dir.display());

    let mut i = 1;
    for asset in assets {
        // if not installed, download release assets.
        // we use a counter to name the assets differently to avoid conflicts in case of multiple assets,
        // which themselves may contain multiple executables.
        let download_to =
            datadirs::get_binary_nest(&cache_dir, repo, &version).join(format!("asset_{}", i));
        let downloaded_file =
            match download_asset(asset.name(), asset.browser_download_url(), &download_to)
                .with_context(|| {
                    format!("Failed to download asset for {} version {}", repo, version)
                }) {
                Ok(file) => file,
                Err(e) => {
                    bail!(e);
                }
            };
        i += 1;

        process_install(&downloaded_file, &download_to, &install_dir, asset.name())
            .with_context(|| format!("Failed to install {} version {}", repo, version))?;

        if clean_cache_dir(&download_to, &cache_dir)? {
            debug!("Cleaned up cache directory: {}", download_to.display());
        }
    }
    info!("{} {} installed successfully.", repo, &version);

    // check if the binaries are in the PATH by checking if poof's bin directory is in PATH
    commands::check::check_if_bin_in_path();
    Ok(())
}

fn process_install(
    downloaded_file: &PathBuf,
    download_to: &PathBuf,
    install_dir: &Path,
    asset_name: &String,
) -> Result<()> {
    // check if downloaded binary is an archive or an executable
    // and proceed accordingly.
    if is_exec_by_magic_number(downloaded_file) {
        debug!("Downloaded file {} is an executable binary.", asset_name);
        let file_name = &downloaded_file
            .file_name()
            .ok_or_else(|| anyhow!("Failed to get filename from {}", downloaded_file.display()))?;
        // Get the stem name trimmed at the first separator for non-archived executable files.
        // This is useful to avoid installing files with names like "mytool-1.0.0" or "mytool-linux-x86_64"
        // and instead use just "mytool", which is how the binary will be used when in PATH.
        let exec_name = get_stem_name_trimmed_at_first_separator(file_name);
        install_binary(downloaded_file, install_dir, &exec_name)
            .with_context(|| format!("Failed to install executable {}", asset_name))?;
    } else {
        // extract executables
        archives::extract_to_dir(downloaded_file, download_to)
            .with_context(|| format!("Failed to extract archive {}", asset_name))?;
        debug!("Extracted {} to {}", asset_name, download_to.display());

        // install executables
        install_binaries(downloaded_file, install_dir).with_context(|| {
            format!("Failed to extract executables from archive {}", asset_name)
        })?;
    }
    Ok(())
}

/// Select the assets to download for the requested software.
/// Returns a tuple of the release and the asset.
/// Returns an error if the release or asset cannot be selected.
pub fn select_assets(repo: &str, tag: Option<&str>) -> Result<(Release, Vec<ReleaseAsset>)> {
    // select assets to download
    let release: Release = get_release(repo, tag)
        .with_context(|| format!("Failed to get release information for {}", repo))?;
    let assets: Vec<ReleaseAsset> = get_assets(&release).with_context(|| {
        format!(
            "Failed to find compatible asset for release {}",
            release.tag_name()
        )
    })?;
    Ok((release, assets))
}

/// Get the installation directory for the requested software.
/// based on repo slug and version.
fn get_install_dir(repo: &str, version: &str) -> Result<PathBuf> {
    let data_dir: PathBuf =
        datadirs::get_data_dir().context("Failed to determine data directory.")?;
    let install_dir: PathBuf = datadirs::get_binary_nest(&data_dir, repo, version);
    Ok(install_dir)
}

/// Prepare the install directory for the requested software.
/// Creates the installation directory if it does not exist.
/// Returns an error if the installation directory cannot be created.
fn prepare_install_dir(install_dir: &PathBuf) -> Result<()> {
    debug!("Preparing install directory: {}", install_dir.display());
    std::fs::create_dir_all(install_dir).with_context(|| {
        format!(
            "Failed to create installation directory {}",
            install_dir.display()
        )
    })?;
    debug!("Created install directory: {}", install_dir.display());
    Ok(())
}

/// Check if the requested software is already installed.
/// Returns true if the software is already installed, false if it should be installed.
/// Returns an error if the installation directory cannot be checked.
fn check_if_installed(install_dir: &Path) -> Result<bool> {
    if !install_dir.exists() {
        // directory does not exist, we can assume requested software
        // is not installed and proceed with installation
        debug!(
            "Install directory does not exist: {}",
            install_dir.display()
        );
        Ok(false)
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
                // directory exists but is empty, proceed with installation
                debug!("Install directory exists but is empty, proceeding.");
                Ok(false) // installation should proceed
            } else {
                // directory exists and is not empty: we assume it's already installed
                warn!(
                    "Version already installed. Check content in {}. Skipping installation.",
                    install_dir.display()
                );
                // we return true to indicate skipping installation
                Ok(true)
            }
        } else {
            // install_dir path exists but it is not a directory.
            // we interpret it as an error to prevent overwriting a user's file.
            bail!(
                "Install path {} exists but is not a directory. Please remove it manually.",
                install_dir.display()
            );
        }
    }
}

fn install_binaries(archive_path: &Path, install_dir: &Path) -> Result<()> {
    // TODO: ensure filesys::find_exec_files_from_extracted_archive returns Result if needed
    // assuming for now it returns Vec<PathBuf> and handles its own errors internally or doesn't fail often
    let execs_to_install: Vec<PathBuf> =
        filesys::find_exec_files_from_extracted_archive(archive_path);

    if execs_to_install.is_empty() {
        // we interpret this as an error
        bail!("No executables found to install. Please check the archive contents.");
    }

    for exec in execs_to_install {
        debug!("Installing executable: {}", exec.display());
        // if we have multiple executables, we install each one.
        // we assume that to have multiple executables, those were in an archive.
        let exec_name = exec
            .file_name()
            .ok_or_else(|| anyhow!("Failed to get filename from {}", exec.display()))?;
        install_binary(&exec, install_dir, &OsString::from(exec_name))
            .with_context(|| format!("Failed to install executable {}", exec.display()))?;
    }
    Ok(())
}

fn install_binary(exec: &PathBuf, install_dir: &Path, exec_stem: &OsString) -> Result<()> {
    let installed_exec = install_dir.join(exec_stem);

    // copy the executable files to the install directory
    filesys::copy_file(exec, &installed_exec).map_err(|e| {
        anyhow!(
            "Failed to copy {} to install dir ({}): {}",
            exec.display(),
            installed_exec.display(),
            e
        )
    })?;

    let bin_dir: PathBuf = datadirs::get_bin_dir().unwrap();

    // make them executable
    // Set executable permissions, platform-specific
    // Note: Windows does not require setting executable permissions
    // TODO: below we have the same issue as in line 166, Result that should
    // be handled. we use the same workaround to ignore the warning
    #[cfg(not(target_os = "windows"))]
    {
        // Make the file executable on Unix-like systems
        filesys::make_executable(&installed_exec);
        // Create a symlink in the bin directory, overwriting existing to default
        // using the new version. This is a UX feature to save the user from having to
        // manually set the default version after installation (most cases).
        let symlink_path = bin_dir.join(exec_stem);
        if let Err(e) = filesys::create_symlink(&installed_exec, &symlink_path, true) {
            warn!(
                    "Failed to create symlink for {}: {}. You may need to manually set the default version.",
                    exec_stem.to_string_lossy(),
                    e
                );
        }
    }
    Ok(())
}

/// Best effort clean up of cache directory.
/// Returns true if the cache directory was deleted, false if it was not.
fn clean_cache_dir(dir: &Path, cache_root: &Path) -> Result<bool> {
    // Resolve and ensure we only delete stuff within the cache directory.
    // Canonicalize both paths to handle symlinked temp paths consistently.
    // Fall back to original paths if canonicalization fails.
    let dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let cache_root = cache_root
        .canonicalize()
        .unwrap_or_else(|_| cache_root.to_path_buf());

    if !dir.starts_with(&cache_root) {
        debug!("Refusing to delete non-cache path: {}", dir.display());
        return Ok(false);
    }

    // Best effort to clean up the cache directory.
    match std::fs::remove_dir_all(&dir) {
        Ok(()) => Ok(true),
        Err(e) => {
            debug!("Failed to delete cache directory {}: {}", dir.display(), e);
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests;
