use anyhow::{Context, Result};
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
    let Some(archive_parent) = archive_path.parent() else {
        return Vec::new();
    };
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
pub fn make_executable(file: &Path) -> Result<()> {
    if !file.is_file() {
        debug!("File {} is not a regular file", file.to_string_lossy());
        return Ok(());
    }
    debug!("Making {} executable", file.to_string_lossy());
    // Unix-like systems require setting executable permissions
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(file)
        .with_context(|| format!("Failed to read metadata for {}", file.display()))?
        .permissions();
    // Add executable bits to current permissions (equivalent to chmod +x)
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(file, perms).with_context(|| {
        format!(
            "Failed to set executable permissions for {}",
            file.display()
        )
    })?;
    debug!("Set executable permissions for {}", file.display());
    Ok(())
}

pub fn copy_file(source: &PathBuf, target: &PathBuf) -> Result<()> {
    debug!(
        "Copying file from {} to {}",
        source.display(),
        target.display()
    );
    std::fs::copy(source, target).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            source.display(),
            target.display()
        )
    })?;
    debug!("File copied successfully");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn create_symlink(source: &PathBuf, target: &PathBuf, remove_existing: bool) -> Result<()> {
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
            std::fs::remove_file(target)
                .with_context(|| format!("Failed to remove existing file {}", target.display()))?;
            debug!("Removed existing symlink {}", target.display());
        } else {
            // If the symlink already exists and we don't want to remove it, skip.
            warn!("Symlink {} already exists. Skipping.", target.display());
            return Ok(());
        }
    }

    // Create a symlink in the target directory pointing to the installed binary.
    std::os::unix::fs::symlink(source, target).with_context(|| {
        format!(
            "Failed to create symlink {} -> {}",
            source.display(),
            target.display()
        )
    })?;
    info!(
        "Symlink created: {} -> {}",
        source.display(),
        target.display()
    );
    Ok(())
}
