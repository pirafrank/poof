//! Main file handling 'install' command

use std::path::{Path, PathBuf};

use crate::{
    commands::{self, download::download_binary},
    core::selector::is_env_compatible,
    files::{archives, datadirs, filesys, magic::is_exec_by_magic_number},
    github::client::{get_asset, get_release},
    utils::semver::SemverStringPrefix,
};
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, info, warn};

pub fn process_install(repo: &str, tag: Option<&str>) -> Result<()> {
    // let config_dir = filesys::get_config_dir().ok_or(libc::ENOENT).unwrap();
    // info!("Config directory: {}", config_dir);
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
        install_binary(&downloaded_file, &install_dir)
            .with_context(|| format!("Failed to install executable {}", binary.name()))?;
    } else {
        // extract binary
        archives::extract_to_dir(&downloaded_file, &download_to).unwrap();
        debug!("Extracted to: {}", download_to.display());

        // install binary
        install_binaries(&downloaded_file, &install_dir).with_context(|| {
            format!(
                "Failed to install binaries for {} version {}",
                repo, version
            )
        })?;
    }

    info!("{} installed successfully.", binary.name());
    commands::check::check_if_bin_in_path();
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
        install_binary(&exec, install_dir)
            .with_context(|| format!("Failed to install executable {}", exec.display()))?;
    }
    Ok(())
}

fn install_binary(exec: &PathBuf, install_dir: &Path) -> Result<()> {
    let bin_dir: PathBuf = datadirs::get_bin_dir().unwrap();
    let file_name = exec
        .file_name()
        .ok_or_else(|| anyhow!("Failed to get filename from {}", exec.display()))?;
    let installed_exec = install_dir.join(file_name);

    // copy the executable files to the install directory
    // TODO: this Result may be an Err variant, which should be handled
    // for now, we just use let _ = to ignore the resulting value
    // but it is rrealy important to handle it
    let _ = filesys::copy_file(exec, &installed_exec);

    // make them executable
    // Set executable permissions, platform-specific
    // Note: Windows does not require setting executable permissions
    // TODO: below we have the same issue as in line 166, Result that should
    // be handled. we use the same workaround to ignore the warning
    #[cfg(not(target_os = "windows"))]
    {
        // Make the file executable on Unix-like systems
        filesys::make_executable(&installed_exec);
        // Create a symlink in the bin directory, NOT overwriting existing
        let symlink_path = bin_dir.join(file_name);
        let _ = filesys::create_symlink(&installed_exec, &symlink_path, false);
    }
    Ok(())
}
