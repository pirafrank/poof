use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::files::datadirs;
use crate::files::filesys;

pub fn set_default(repo: &str, version: &str) -> Result<()> {
    let data_dir: PathBuf = datadirs::get_data_dir()
        .context("Failed to determine data directory")?;
    let install_dir: PathBuf = datadirs::get_binary_nest(&data_dir, repo, version);
    
    if !install_dir.exists() {
        bail!(
            "Version {} of repository '{}' is not installed. Please install it first using 'poof install {}'",
            version, repo, repo
        );
    }
    
    // Get the bin directory
    let bin_dir: PathBuf = datadirs::get_bin_dir()
        .context("Failed to determine bin directory")?;
    
    // Process each binary in wanted_dir
    let exec_files = filesys::find_exec_files_in_dir(&install_dir);
    if exec_files.is_empty() {
        bail!(
            "No executable files found in installation directory for {} version {}",
            repo, version
        );
    }
    
    for path in exec_files {
        // Skip non-executable files (they all should be since they have
        // been installed, but just in case).
        // Note: Windows does not require setting executable permissions
        #[cfg(not(target_os = "windows"))]
        {
            if !filesys::is_executable(&path) {
                continue;
            }
            // Get exec filename
            let file_name = match path.file_name() {
                Some(name) => name,
                None => continue,
            };
            // make exec available in PATH, overwriting any existing symlink
            let symlink_path = bin_dir.join(file_name);
            filesys::create_symlink(&path, &symlink_path, true)
                .map_err(|e| anyhow::anyhow!(e))
                .with_context(|| format!(
                    "Failed to create symlink for {} from {} to {}",
                    file_name.to_string_lossy(),
                    path.display(),
                    symlink_path.display()
                ))?;
        }
    }
    Ok(())
}
