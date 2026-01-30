use log::{debug, warn};
use std::path::{Path, PathBuf};

use crate::files::magic::is_exec_by_magic_number;
use crate::files::utils::strip_supported_extensions;

pub fn find_exec_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                // check criteria to determine if a file is a binary
                // 1. Check if the file is a regular file
                // 2. Check if the file is an executable by checking the magic number
                if file_type.is_file() && is_exec_by_magic_number(&entry.path()) {
                    result.push(entry.path());
                }
            }
        }
    }
    result
}

pub fn find_exec_files_from_extracted_archive(archive_path: &Path) -> Vec<PathBuf> {
    let archive_parent = archive_path.parent().unwrap();
    // Get the filename without the extension
    // and create the path of a directory with the same name as the archive, minus the extension.
    // If it exists, we will search for executables in that directory.
    // If it doesn't exist, we will search for executables in the parent directory.
    // This is useful for archives that contain a directory with the same name as the archive.
    let filename_no_ext_str = strip_supported_extensions(archive_path);
    let dir = archive_parent.join(filename_no_ext_str);
    if dir.exists() {
        find_exec_files_in_dir(&dir)
    } else {
        find_exec_files_in_dir(&PathBuf::from(archive_parent))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn is_executable(path: &PathBuf) -> bool {
    // Check if the file is executable
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.is_file() {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
pub fn make_executable(file: &Path) {
    if !file.is_file() {
        debug!("File {} is not a regular file", file.to_string_lossy());
        return;
    }
    debug!("Making {} executable", file.to_string_lossy());
    // Unix-like systems require setting executable permissions
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(file).unwrap().permissions();
    // Add executable bits to current permissions (equivalent to chmod +x)
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(file, perms).unwrap();
    debug!("Set executable permissions for {}", file.display());
}

pub fn copy_file(source: &PathBuf, target: &PathBuf) -> Result<(), String> {
    debug!(
        "Copying file from {} to {}",
        source.display(),
        target.display()
    );
    if let Err(e) = std::fs::copy(source, target) {
        let e_msg = format!(
            "Error copying {} to {}: {}",
            source.display(),
            target.display(),
            e
        );
        return Err(e_msg);
    };
    debug!("File copied successfully");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn create_symlink(
    source: &PathBuf,
    target: &PathBuf,
    remove_existing: bool,
) -> Result<(), String> {
    use log::info;

    let msg = if remove_existing { "" } else { " NOT" };
    debug!(
        "Creating symlink {} -> {},{} removing existing",
        source.display(),
        target.display(),
        msg
    );
    if target.exists() {
        if remove_existing {
            if let Err(e) = std::fs::remove_file(target) {
                return Err(format!("Cannot remove {}, Error: {}", target.display(), e));
            }
            debug!("Removed existing symlink {}", target.display());
        } else {
            // If the symlink already exists and we don't want to remove it, skip.
            warn!("Symlink {} already exists. Skipping.", target.display());
            return Ok(());
        }
    }

    // Create a symlink in the target directory pointing to the installed binary.
    match std::os::unix::fs::symlink(source, target) {
        Ok(_) => {
            info!(
                "Symlink created: {} -> {}",
                source.display(),
                target.display()
            );
        }
        Err(e) => {
            let e_msg = format!(
                "Error creating symlink {} -> {}: {}",
                source.display(),
                target.display(),
                e
            );
            return Err(e_msg);
        }
    }
    Ok(())
}

/// Check if a symlink is broken.
/// Returns true if the symlink is broken, false otherwise.
pub fn is_broken_symlink(path: &Path) -> std::io::Result<bool> {
    // this uses try_exists to check if the target exists.
    // which is more efficient than reading the symlink target and checking if it exists.
    // try_exists has been added in Rust 1.63.0.
    // doc: https://doc.rust-lang.org/std/path/struct.Path.html#method.try_exists
    let sym_meta = std::fs::symlink_metadata(path)?;

    if sym_meta.is_symlink() {
        Ok(!path.try_exists()?)
    } else {
        Ok(false)
    }
}
