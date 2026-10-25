//! File system helpers for locating, copying, and symlinking executables.

use log::debug;
use std::path::{Path, PathBuf};

use crate::files::magic::{is_exec_by_magic_number, is_exec_for_current_arch};

/// Return all executable files found inside `dir` (recursively).
///
/// A file is considered executable when inner checks on the file header
/// return `true` for it. Directories and symlinks are ignored.
///
/// If `deep` is `true`, the function will check if the file is an executable
/// by checking the magic number AND the architecture.
/// If `deep` is `false`, it will only check the magic number.
pub fn find_exec_files_in_dir(dir: &Path, deep: bool) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![dir.to_path_buf()];

    while let Some(dir) = stack.pop() {
        // move to next iteration if the directory does not exist or is not a directory
        if !dir.exists() || !dir.is_dir() {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(e) => {
                debug!("Skipping unreadable directory {}: {}", dir.display(), e);
                continue;
            }
        };
        for entry in entries.flatten() {
            // going entry.file_type avoid extra stat sys call
            if let Ok(file_type) = entry.file_type() {
                // this works for small directories trees like the ones we have in
                // the install directory or inside an extracted archive.
                // if it's a directory, we add it to the stack to explore it recursively,
                // otherwise we check if the file is a binary and add it to the result list.
                // check criteria to determine if a file is a binary
                // 1. Check if the file is a regular file
                // 2. Check if the file is an executable by checking the magic number
                if file_type.is_dir() {
                    stack.push(entry.path());
                } else if file_type.is_file()
                    && ((deep && is_exec_for_current_arch(&entry.path()).unwrap_or(false))
                        || (!deep && is_exec_by_magic_number(&entry.path())))
                {
                    let s = entry.path().display().to_string();
                    debug!("Found executable file: {}", s);
                    result.push(entry.path());
                } // else we don't care
            }
        }
    }

    result
}

/// Return `true` when `path` is a regular file with at least one executable bit set (Unix only).
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

/// Add executable permission bits to `file` (equivalent to `chmod +x`) (Unix only).
///
/// Has no effect if `file` is not a regular file.
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

/// Copy `source` to `target`, returning a descriptive error string on failure.
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

/// Replace a symlink only when its current target belongs to `repo_root`.
#[cfg(not(target_os = "windows"))]
pub fn create_repo_symlink(
    source: &PathBuf,
    target: &PathBuf,
    repo_root: &Path,
) -> Result<(), String> {
    match std::fs::symlink_metadata(target) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            let link_target = std::fs::read_link(target)
                .map_err(|error| format!("Cannot read {}: {error}", target.display()))?;
            let resolved_target = if link_target.is_absolute() {
                link_target
            } else {
                target
                    .parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join(link_target)
            };
            let resolved_target = resolved_target
                .canonicalize()
                .map_err(|error| format!("Cannot resolve {}: {error}", target.display()))?;
            let repo_root = repo_root
                .canonicalize()
                .map_err(|error| format!("Cannot resolve repository directory: {error}"))?;
            if !resolved_target.starts_with(&repo_root) {
                return Err(format!(
                    "Refusing to replace symlink not owned by repository: {}",
                    target.display()
                ));
            }
            std::fs::remove_file(target)
                .map_err(|error| format!("Cannot remove {}: {error}", target.display()))?;
        }
        Ok(_) => {
            return Err(format!(
                "Refusing to replace non-symlink {}",
                target.display()
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(format!("Cannot inspect {}: {error}", target.display())),
    }
    std::os::unix::fs::symlink(source, target)
        .map_err(|error| format!("Cannot create symlink {}: {error}", target.display()))
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

#[cfg(test)]
mod tests;
