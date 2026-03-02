//! File system helpers for locating, copying, and symlinking executables.

use log::{debug, warn};
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

/// Create a Unix symlink at `target` pointing to `source` (Unix only).
///
/// When `remove_existing` is `true` any file already at `target` is deleted
/// before the symlink is created. When it is `false` and `target` already
/// exists the operation is skipped with a warning.
#[cfg(not(target_os = "windows"))]
pub fn create_symlink(
    source: &PathBuf,
    target: &PathBuf,
    remove_existing: bool,
) -> Result<(), String> {
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
            debug!(
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    fn write_tmp_file(dir: &TempDir, name: &str, bytes: &[u8]) -> PathBuf {
        let path = dir.path().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(bytes).unwrap();
        path
    }

    // *** copy_file **********************************************************

    #[test]
    fn test_copy_file_success() {
        let dir = TempDir::new().unwrap();
        let src = write_tmp_file(&dir, "src.bin", b"hello");
        let dst = dir.path().join("dst.bin");
        let result = copy_file(&src, &dst);
        assert!(result.is_ok());
        assert_eq!(std::fs::read(&dst).unwrap(), b"hello");
    }

    #[test]
    fn test_copy_file_missing_source() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("does_not_exist");
        let dst = dir.path().join("dst.bin");
        let result = copy_file(&src, &dst);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Error copying"));
    }

    // *** is_broken_symlink **************************************************

    #[test]
    fn test_is_broken_symlink_regular_file() {
        let f = NamedTempFile::new().unwrap();
        assert!(!is_broken_symlink(f.path()).unwrap());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_is_broken_symlink_with_valid_symlink() {
        let dir = TempDir::new().unwrap();
        let target = write_tmp_file(&dir, "real.bin", b"data");
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        assert!(!is_broken_symlink(&link).unwrap());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_is_broken_symlink_with_broken_symlink() {
        let dir = TempDir::new().unwrap();
        let ghost = dir.path().join("ghost");
        let link = dir.path().join("broken_link");
        std::os::unix::fs::symlink(&ghost, &link).unwrap();
        // ghost was never created, so the link is broken
        assert!(is_broken_symlink(&link).unwrap());
    }

    // *** Unix-only functions *************************************************

    #[cfg(not(target_os = "windows"))]
    mod unix {
        use super::*;
        use std::os::unix::fs::PermissionsExt;

        #[test]
        fn test_is_executable_with_exec_bit_set() {
            let dir = TempDir::new().unwrap();
            let path = write_tmp_file(&dir, "exec.bin", b"data");
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&path, perms).unwrap();
            assert!(is_executable(&path));
        }

        #[test]
        fn test_is_executable_without_exec_bit() {
            let dir = TempDir::new().unwrap();
            let path = write_tmp_file(&dir, "noexec.bin", b"data");
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(perms.mode() & !0o111);
            std::fs::set_permissions(&path, perms).unwrap();
            assert!(!is_executable(&path));
        }

        #[test]
        fn test_make_executable_sets_exec_bits() {
            let dir = TempDir::new().unwrap();
            let path = write_tmp_file(&dir, "target.bin", b"data");
            // Strip exec bits first
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(perms.mode() & !0o111);
            std::fs::set_permissions(&path, perms).unwrap();
            assert!(!is_executable(&path));

            make_executable(&path);

            assert!(is_executable(&path));
        }

        #[test]
        fn test_make_executable_on_directory_is_noop() {
            let dir = TempDir::new().unwrap();
            // Should not panic when called with a directory path
            make_executable(dir.path());
        }

        #[test]
        fn test_create_symlink_creates_link() {
            let dir = TempDir::new().unwrap();
            let target = write_tmp_file(&dir, "real.bin", b"data");
            let link = dir.path().join("link");
            let result = create_symlink(&target, &link, false);
            assert!(result.is_ok());
            assert!(link.exists());
        }

        #[test]
        fn test_create_symlink_skips_when_exists_and_no_remove() {
            let dir = TempDir::new().unwrap();
            let target = write_tmp_file(&dir, "real.bin", b"data");
            let link = dir.path().join("link");
            create_symlink(&target, &link, false).unwrap();
            // Second call with remove_existing=false should silently skip
            let result = create_symlink(&target, &link, false);
            assert!(result.is_ok());
        }

        #[test]
        fn test_create_symlink_replaces_when_remove_existing() {
            let dir = TempDir::new().unwrap();
            let target1 = write_tmp_file(&dir, "real1.bin", b"v1");
            let target2 = write_tmp_file(&dir, "real2.bin", b"v2");
            let link = dir.path().join("link");
            create_symlink(&target1, &link, false).unwrap();
            let result = create_symlink(&target2, &link, true);
            assert!(result.is_ok());
            // Link should now point to target2
            let contents = std::fs::read(&link).unwrap();
            assert_eq!(contents, b"v2");
        }
    }

    // *** find_exec_files_in_dir *********************************************

    #[test]
    fn test_find_exec_files_in_dir_empty_dir() {
        let dir = TempDir::new().unwrap();
        let found = find_exec_files_in_dir(dir.path(), false);
        assert!(found.is_empty());
    }

    #[test]
    fn test_find_exec_files_in_dir_nonexistent_path() {
        let path = std::path::Path::new("/tmp/poof_no_such_dir_filesys_test_xyz999");
        let found = find_exec_files_in_dir(path, false);
        assert!(found.is_empty());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_find_exec_files_in_dir_finds_shebang_file() {
        let dir = TempDir::new().unwrap();
        // A shebang file is recognised as executable by is_exec_by_magic_number
        let script = write_tmp_file(&dir, "script.sh", b"#!/bin/sh\necho hi\n");
        let _non_exec = write_tmp_file(&dir, "data.txt", b"plain text");
        let found = find_exec_files_in_dir(dir.path(), false);
        assert!(found.contains(&script));
        assert!(!found.contains(&dir.path().join("data.txt")));
    }
}
