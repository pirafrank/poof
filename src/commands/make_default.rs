use std::path::PathBuf;

use log::error;

use crate::core::filesys;
use crate::datadirs;

pub fn set_default(repo: &str, version: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data_dir: PathBuf = datadirs::get_data_dir().ok_or(libc::ENOENT).unwrap();
    let install_dir: PathBuf = datadirs::get_binary_nest(&data_dir, repo, version);
    if !install_dir.exists() {
        error!(
            "Version {} of repository '{}' not installed. Quitting.",
            version, repo
        );
        std::process::exit(110);
    }
    // Get the bin directory
    let bin_dir: PathBuf = datadirs::get_bin_dir().ok_or(libc::ENOENT).unwrap();
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
            let file_name = match path.file_name() {
                Some(name) => name,
                None => continue,
            };
            // make exec available in PATH, overwriting any existing symlink
            let symlink_path = bin_dir.join(file_name);
            filesys::create_symlink(&path, &symlink_path, true)?;
        }
    }
    Ok(())
}
