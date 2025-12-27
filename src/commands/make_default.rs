use std::path::PathBuf;

use anyhow::{Context, Result};
use log::error;

use crate::files::datadirs;
use crate::files::filesys;
use crate::files::utils::find_similar_repo;

fn get_installed_dir(repo: &str, version: &str) -> Result<PathBuf> {
    let data_dir = datadirs::get_data_dir().context("Failed to get data directory")?;

    let installed_repo_dir = datadirs::get_versions_nest(&data_dir, repo);
    if !installed_repo_dir.exists() {
        // Try fuzzy finding a similar named installed repository
        if let Some(similar_repo) = find_similar_repo(&data_dir, repo) {
            error!(
                "It looks like '{}' is not installed. Did you mean: {}",
                repo, similar_repo
            );
        } else {
            error!("It looks like '{}' is not installed. Typo?", repo);
        }
        error!("Check installed binaries using 'list' command.");
        std::process::exit(110);
    }

    let installed_version_dir = datadirs::get_binary_nest(&data_dir, repo, version);
    if !installed_version_dir.exists() {
        error!(
            "Version {} of repository '{}' is not installed. Typo?",
            version, repo
        );
        error!("Check installed versions using 'list' command.");
        std::process::exit(110);
    }

    Ok(installed_version_dir)
}

pub fn set_default(repo: &str, version: &str) -> Result<()> {
    // Get the installed directory for the specified repo and version
    let install_dir = get_installed_dir(repo, version)?;
    // Get the bin directory
    let bin_dir = datadirs::get_bin_dir().context("Failed to get bin directory")?;

    // Process each binary in wanted_dir
    for path in filesys::find_exec_files_in_dir(&install_dir) {
        // Skip non-executable files (they all should be since they have
        // been installed, but just in case).
        // Note: Windows does not require setting executable permissions
        #[cfg(not(target_os = "windows"))]
        {
            if !filesys::is_executable(&path) {
                continue;
            }
            // Get exec filename
            let Some(file_name) = path.file_name() else {
                continue;
            };
            // make exec available in PATH, overwriting any existing symlink
            let symlink_path = bin_dir.join(file_name);
            filesys::create_symlink(&path, &symlink_path, true).with_context(|| {
                format!(
                    "Failed to create symlink from {} to {}",
                    path.display(),
                    symlink_path.display()
                )
            })?;
        }
    }
    Ok(())
}
