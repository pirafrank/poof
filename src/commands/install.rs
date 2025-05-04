//! Main file handling 'install' command

use std::path::{Path, PathBuf};

use crate::{
    archives,
    commands::{self, download::download_binary},
    datadirs, filesys,
    github::client::{get_asset, get_release},
    selector::is_env_compatible,
    semver_utils::SemverStringPrefix,
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
    // TODO: refactor get_release and get_asset to return Result
    let release = get_release(repo, tag)
        .with_context(|| format!("Failed to get release information for {}", repo))?; 
    let binary = get_asset(&release, is_env_compatible) 
        .with_context(|| format!("Failed to find compatible asset for release {}", release.tag_name()))?; 
    let version: String = release.tag_name().strip_v();
    let download_to = datadirs::get_binary_nest(&cache_dir, repo, &version);

    download_binary(binary.name(), binary.browser_download_url(), &download_to)
        .with_context(|| format!("Failed to download binary {} version {}", repo, version))?;

    // extract binary
    let archive_path = download_to.join(binary.name());
    // TODO: refactor archives::extract_to_dir_depending_on_content_type to return Result
    archives::extract_to_dir_depending_on_content_type(
        binary.content_type(),
        &archive_path,
        &download_to,
    )
    .unwrap();

    debug!("Extracted to: {}", download_to.display());

    // install binary
    install_binaries(&archive_path, repo, &version).with_context(|| {
        format!(
            "Failed to install binaries for {} version {}",
            repo, version
        )
    })?;

    info!("{} installed successfully.", binary.name());
    commands::check::check_if_bin_in_path();
    Ok(())
}

// Result<bool>: true = proceed, false = skip/already done
// in this way we eliminate the std::process::exit(0)
fn prepare_install_dir(install_dir: &PathBuf) -> Result<bool> {
    debug!("Installing to: {}", install_dir.display());
    // Create the installation directory if it doesn't exist
    if !install_dir.exists() {
        // directory does not exist, create it
        std::fs::create_dir_all(install_dir).with_context(|| {
            format!(
                "Failed to create installation directory {}",
                install_dir.display()
            )
        })?;
        debug!("Created install directory: {}", install_dir.display());
        Ok(true)
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
                Ok(true) // installation should proceed
            } else {
                // directory exists and is not empty: we assume it's already installed
                warn!(
                    "Version already installed. Check content in {}. Skipping installation.",
                    install_dir.display()
                );
                // we return false to indicate skipping
                Ok(false)
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

fn install_binaries(archive_path: &Path, repo: &str, version: &str) -> Result<()> {
    let data_dir: PathBuf =
        datadirs::get_data_dir().context("Failed to determine data directory.")?;
    debug!("Data directory: {}", data_dir.display());
    let install_dir: PathBuf = datadirs::get_binary_nest(&data_dir, repo, version);

    // prepare install dir, check the boolean result
    let should_proceed = prepare_install_dir(&install_dir)
        .with_context(|| format!("Failed to prepare install directory for {}", repo))?;

    if !should_proceed {
        // prepare_install_dir already warned, just return Ok
        info!(
            "Skipping installation as version {} for {} seems already installed.",
            version, repo
        );
        return Ok(());
    }

    let bin_dir: PathBuf = datadirs::get_bin_dir().unwrap();

    // TODO: ensure filesys::find_exec_files_from_extracted_archive returns Result if needed
    // assuming for now it returns Vec<PathBuf> and handles its own errors internally or doesn't fail often
    let execs_to_install: Vec<PathBuf> =
        filesys::find_exec_files_from_extracted_archive(archive_path);

    if execs_to_install.is_empty() {
        // we interpret this as an error
        bail!(
            "No executable found in the extracted archive at {}",
            archive_path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "unknown location".to_string()) // fallback message
        );
    }

    for exec in execs_to_install {
        install_binary(&exec, &install_dir, &bin_dir)
            .with_context(|| format!("Failed to install executable {}", exec.display()))?;
    }
    Ok(())
}

fn install_binary(exec: &PathBuf, install_dir: &Path, bin_dir: &Path) -> Result<()> {
    let file_name = exec
        .file_name()
        .ok_or_else(|| anyhow!("Failed to get filename from {}", exec.display()))?;
    let installed_exec = install_dir.join(file_name);

    // copy the executable files to the install directory
    // TODO: filesys::copy_file should return anyhow::Result
    filesys::copy_file(exec, &installed_exec);

    // make them executable
    // Set executable permissions, platform-specific
    // Note: Windows does not require setting executable permissions
    #[cfg(not(target_os = "windows"))]
    {
        // Make the file executable on Unix-like systems
        filesys::make_executable(&installed_exec);
        // Create a symlink in the bin directory, NOT overwriting existing
        let symlink_path = bin_dir.join(file_name);
        filesys::create_symlink(&installed_exec, &symlink_path, false);
    }
    Ok(())
}