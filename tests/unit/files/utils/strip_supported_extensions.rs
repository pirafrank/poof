use poof::files::utils::strip_supported_extensions;
use std::path::PathBuf;

// Test all supported multi-part extensions
#[test]
fn test_strip_tar_gz_extension() {
    let path = PathBuf::from("archive.tar.gz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_tgz_extension() {
    let path = PathBuf::from("archive.tgz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_tar_xz_extension() {
    let path = PathBuf::from("archive.tar.xz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_txz_extension() {
    let path = PathBuf::from("archive.txz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_tar_bz2_extension() {
    let path = PathBuf::from("archive.tar.bz2");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_tbz_extension() {
    let path = PathBuf::from("archive.tbz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_tbz2_extension() {
    let path = PathBuf::from("archive.tbz2");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

// Test single extensions
#[test]
fn test_strip_zip_extension() {
    let path = PathBuf::from("archive.zip");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_tar_extension() {
    let path = PathBuf::from("archive.tar");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_gz_extension() {
    let path = PathBuf::from("archive.gz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_xz_extension() {
    let path = PathBuf::from("archive.xz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_bz2_extension() {
    let path = PathBuf::from("archive.bz2");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

// Test with full paths
#[test]
fn test_strip_extension_with_full_path() {
    let path = PathBuf::from("/path/to/archive.tar.gz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_extension_with_relative_path() {
    let path = PathBuf::from("./downloads/archive.zip");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

// Test complex filenames
#[test]
fn test_strip_extension_with_multiple_dots() {
    let path = PathBuf::from("my.app.name.tar.gz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "my.app.name");
}

#[test]
fn test_strip_extension_real_world_binary_name() {
    let path = PathBuf::from("ripgrep-13.0.0-x86_64-unknown-linux-musl.tar.gz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "ripgrep-13.0.0-x86_64-unknown-linux-musl");
}

// Test unsupported extensions - should fall back to file_stem
#[test]
fn test_strip_unsupported_extension() {
    let path = PathBuf::from("archive.rar");
    let result = strip_supported_extensions(&path);
    // Should fall back to file_stem which removes .rar
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_unsupported_extension_7z() {
    let path = PathBuf::from("archive.7z");
    let result = strip_supported_extensions(&path);
    // Should fall back to file_stem which removes .7z
    assert_eq!(result, "archive");
}

// Test files without extensions, like binaries
#[test]
fn test_strip_no_extension() {
    let path = PathBuf::from("archive");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "archive");
}

#[test]
fn test_strip_no_extension_with_path() {
    let path = PathBuf::from("/path/to/binary");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, "binary");
}

// Test hidden files
#[test]
fn test_strip_hidden_file_with_extension() {
    let path = PathBuf::from(".hidden.tar.gz");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, ".hidden");
}

#[test]
fn test_strip_hidden_file_no_extension() {
    let path = PathBuf::from(".hidden");
    let result = strip_supported_extensions(&path);
    assert_eq!(result, ".hidden");
}

// Test edge cases
#[test]
fn test_strip_double_extension() {
    let path = PathBuf::from("archive.tar.gz.tar.gz");
    let result = strip_supported_extensions(&path);
    // Should only strip the last .tar.gz
    assert_eq!(result, "archive.tar.gz");
}
