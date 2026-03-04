use crate::files::utils::is_checksum_file;

#[test]
fn test_is_checksum_file_checksum_txt() {
    assert!(is_checksum_file("checksum.txt"));
}

#[test]
fn test_is_checksum_file_checksums_txt() {
    assert!(is_checksum_file("checksums.txt"));
}

#[test]
fn test_is_checksum_file_sha256() {
    assert!(is_checksum_file("tool-1.0.sha256"));
}

#[test]
fn test_is_checksum_file_sha256sum() {
    assert!(is_checksum_file("tool-1.0.sha256sum"));
}

#[test]
fn test_is_checksum_file_sha1() {
    assert!(is_checksum_file("tool-1.0.sha1"));
}

#[test]
fn test_is_checksum_file_sha1sum() {
    assert!(is_checksum_file("tool-1.0.sha1sum"));
}

#[test]
fn test_is_checksum_file_md5() {
    assert!(is_checksum_file("tool-1.0.md5"));
}

#[test]
fn test_is_checksum_file_md5sum() {
    assert!(is_checksum_file("tool-1.0.md5sum"));
}

#[test]
fn test_is_checksum_file_sha512() {
    assert!(is_checksum_file("tool-1.0.sha512"));
}

#[test]
fn test_is_checksum_file_sha512sum() {
    assert!(is_checksum_file("tool-1.0.sha512sum"));
}

#[test]
fn test_is_checksum_file_crc32() {
    assert!(is_checksum_file("tool-1.0.crc32"));
}

#[test]
fn test_is_checksum_file_crc64() {
    assert!(is_checksum_file("tool-1.0.crc64"));
}

#[test]
fn test_is_checksum_file_crc() {
    assert!(is_checksum_file("tool-1.0.crc"));
}

#[test]
fn test_is_checksum_file_sfv() {
    assert!(is_checksum_file("tool-1.0.sfv"));
}

#[test]
fn test_is_checksum_file_case_insensitive() {
    assert!(is_checksum_file("TOOL.SHA256"));
}

#[test]
fn test_is_checksum_file_non_matching() {
    assert!(!is_checksum_file("tool-1.0.tar.gz"));
}
