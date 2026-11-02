use crate::files::archives::{copy_limited_with_limit, validate_tar_limits};
use std::io::Cursor;

#[test]
fn test_copy_limited_removes_oversized_output() {
    let temp_dir = tempfile::tempdir().unwrap();
    let output = temp_dir.path().join("output");

    let result = copy_limited_with_limit(&mut Cursor::new(b"too large"), &output, 4);

    assert!(result.is_err());
    assert!(!output.exists());
}

#[test]
fn test_validate_tar_limits_rejects_oversized_entry() {
    let temp_dir = tempfile::tempdir().unwrap();
    let archive_path = temp_dir.path().join("oversized.tar");
    let mut header = tar::Header::new_gnu();
    header.set_path("large-file").unwrap();
    header.set_size(1024 * 1024 * 1024 + 1);
    header.set_cksum();
    std::fs::write(&archive_path, header.as_bytes()).unwrap();

    let result = validate_tar_limits(std::fs::File::open(archive_path).unwrap());

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("extraction limits"));
}
