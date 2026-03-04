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
