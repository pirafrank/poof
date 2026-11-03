//! Tests for TAR extraction settings.

use std::io::Cursor;
use std::path::Path;

use tar::{Builder, Header};
use tempfile::TempDir;

use crate::files::archives::unpack_tar;

#[test]
#[serial_test::serial]
fn unpack_tar_does_not_preserve_special_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let extract_path = temp_dir.path().join("extracted");
    let archive = create_tar_with_file("file.txt", b"content", 0o4777);

    unpack_tar(Cursor::new(archive), &extract_path).unwrap();

    let extracted_file = extract_path.join("file.txt");
    assert_eq!(std::fs::read(&extracted_file).unwrap(), b"content");
    assert_special_permissions_not_preserved(&extracted_file);
}

#[cfg(target_os = "linux")]
#[test]
fn unpack_tar_ignores_linux_extended_attributes() {
    assert_tar_xattr_is_ignored("user.poof_test");
}

#[cfg(target_os = "macos")]
#[test]
fn unpack_tar_ignores_macos_extended_attributes() {
    assert_tar_xattr_is_ignored("com.poof.test");
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn assert_tar_xattr_is_ignored(attribute_name: &str) {
    use std::ffi::CString;

    let temp_dir = TempDir::new().unwrap();
    let extract_path = temp_dir.path().join("extracted");
    let pax_key = format!("SCHILY.xattr.{attribute_name}");
    let archive = create_tar_with_xattr("file.txt", b"content", &pax_key, "archived");
    unpack_tar(Cursor::new(archive), &extract_path).unwrap();

    let file_path =
        CString::new(extract_path.join("file.txt").as_os_str().as_encoded_bytes()).unwrap();
    let name = CString::new(attribute_name).unwrap();
    #[cfg(target_os = "linux")]
    let result =
        unsafe { libc::getxattr(file_path.as_ptr(), name.as_ptr(), std::ptr::null_mut(), 0) };
    #[cfg(target_os = "macos")]
    let result = unsafe {
        libc::getxattr(
            file_path.as_ptr(),
            name.as_ptr(),
            std::ptr::null_mut(),
            0,
            0,
            0,
        )
    };
    assert_eq!(result, -1, "the archived extended attribute was restored");
}

#[cfg(unix)]
fn assert_special_permissions_not_preserved(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    assert_eq!(
        std::fs::metadata(path).unwrap().permissions().mode() & 0o4000,
        0
    );
}

#[cfg(not(unix))]
fn assert_special_permissions_not_preserved(_path: &Path) {}

fn create_tar_with_file(path: &str, contents: &[u8], mode: u32) -> Vec<u8> {
    let mut builder = Builder::new(Vec::new());
    let mut header = Header::new_gnu();
    header.set_path(path).unwrap();
    header.set_size(contents.len() as u64);
    header.set_mode(mode);
    header.set_cksum();
    builder.append(&header, contents).unwrap();
    builder.into_inner().unwrap()
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn create_tar_with_xattr(path: &str, contents: &[u8], key: &str, value: &str) -> Vec<u8> {
    let mut builder = Builder::new(Vec::new());
    builder
        .append_pax_extensions([(key, value.as_bytes())])
        .unwrap();
    let mut header = Header::new_gnu();
    header.set_path(path).unwrap();
    header.set_size(contents.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append(&header, contents).unwrap();
    builder.into_inner().unwrap()
}
