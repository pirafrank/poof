//! Main file handling 'install' command

use std::path::{Path, PathBuf};

use crate::selector::is_env_compatible;
use log::{debug, error, info, warn};

use crate::{
    archives,
    commands::{self, download::download_binary},
    filesys,
    github::client::{get_asset, get_release},
    semver_utils::SemverStringPrefix,
};

pub fn process_install(repo: &str, tag: Option<&str>) {
    // let config_dir = filesys::get_config_dir().ok_or(libc::ENOENT).unwrap();
    // info!("Config directory: {}", config_dir);
    let cache_dir: PathBuf = filesys::get_cache_dir().ok_or(libc::ENOENT).unwrap();
    debug!("Cache directory: {}", cache_dir.display());

    // download binary
    let release = get_release(repo, tag);
    let binary = get_asset(&release, is_env_compatible);
    let version: String = release.tag_name().strip_v();
    let download_to = filesys::get_binary_nest(&cache_dir, repo, &version);
    download_binary(binary.name(), binary.browser_download_url(), &download_to);

    // extract binary
    let archive_path = download_to.join(binary.name());
    if let Err(e) = archives::extract_to_dir_depending_on_content_type(
        binary.content_type(),
        &archive_path,
        &download_to,
    ) {
        error!("Failed to extract {}: {}", archive_path.display(), e);
        std::process::exit(108);
    }
    debug!("Extracted to: {}", download_to.display());

    // install binary
    install_binaries(&archive_path, repo, &version);
    info!("{} installed successfully.", binary.name());
    commands::check::check_if_bin_in_path();
    std::process::exit(0);
}

fn prepare_install_dir(install_dir: &PathBuf) {
    debug!("Installing to: {}", install_dir.display());
    // Create the installation directory if it doesn't exist
    if !install_dir.exists() {
        std::fs::create_dir_all(install_dir).unwrap();
    } else if install_dir.is_dir() && install_dir.read_dir().unwrap().count() > 0 {
        // Check if the directory is not empty
        // If it is not empty, warn the user and exit
        warn!(
            "Version is already installed. Check content in {} dir.",
            install_dir.display()
        );
        warn!("If you want to reinstall, please remove the directory first.");
        std::process::exit(0);
    } // else overwrite empty dir of possibly left-over dumb file with dir name
}

fn install_binaries(archive_path: &Path, repo: &str, version: &str) {
    let data_dir: PathBuf = filesys::get_data_dir().ok_or(libc::ENOENT).unwrap();
    debug!("Data directory: {}", data_dir.display());
    let install_dir: PathBuf = filesys::get_binary_nest(&data_dir, repo, version);
    prepare_install_dir(&install_dir);

    let execs_to_install: Vec<PathBuf> =
        filesys::find_exec_files_from_extracted_archive(archive_path);
    for exec in execs_to_install {
        // copy the executable files to the install directory
        // make them executable
        // and create symlinks to them in the bin directory
        let bin_dir: PathBuf = filesys::get_bin_dir().ok_or(libc::ENOENT).unwrap();
        if let Err(e) = install_binary(&exec, &install_dir, &bin_dir) {
            error!("Failed to install {}: {}", exec.display(), e);
            std::process::exit(109);
        }
    }
}

fn install_binary(
    exec: &PathBuf,
    install_dir: &Path,
    bin_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = exec.file_name().unwrap();
    let installed_exec = install_dir.join(file_name);
    // copy the executable files to the install directory
    filesys::copy_file(exec, &installed_exec)?;
    // make them executable
    // Set executable permissions, platform-specific
    // Note: Windows does not require setting executable permissions
    #[cfg(not(target_os = "windows"))]
    {
        // Make the file executable on Unix-like systems
        filesys::make_executable(&installed_exec);
        // Create a symlink in the bin directory, NOT overwriting existing
        let symlink_path = bin_dir.join(file_name);
        filesys::create_symlink(&installed_exec, &symlink_path, false)?;
    }
    Ok(())
}
