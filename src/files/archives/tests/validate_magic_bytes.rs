//! Unit tests for archive functions
//! Tests archive format detection and validation

use crate::files::magic::{
    BZIP2_MAGIC, GZIP_MAGIC, SEVENZ_MAGIC, TAR_MAGIC, TAR_MAGIC_OFFSET, XZ_MAGIC, ZIP_MAGIC,
};
use crate::models::binary_container::BinaryContainer;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

use super::common::*;
use crate::files::archives::get_validated_archive_format;

// ============================================================================
// Tests for valid archives with matching extension and magic bytes
// ============================================================================

#[test]
fn test_valid_zip_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.zip");
    create_file_with_magic(&file_path, ZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Zip);
}

#[test]
fn test_valid_gzip_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.gz");
    create_file_with_magic(&file_path, GZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Gz);
}

#[test]
fn test_valid_tar_gz_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.tar.gz");
    create_file_with_magic(&file_path, GZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarGz);
}

#[test]
fn test_valid_tgz_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.tgz");
    create_file_with_magic(&file_path, GZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarGz);
}

#[test]
fn test_valid_xz_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.xz");
    create_file_with_magic(&file_path, XZ_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Xz);
}

#[test]
fn test_valid_tar_xz_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.tar.xz");
    create_file_with_magic(&file_path, XZ_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarXz);
}

#[test]
fn test_valid_txz_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txz");
    create_file_with_magic(&file_path, XZ_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarXz);
}

#[test]
fn test_valid_bzip2_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.bz2");
    create_file_with_magic(&file_path, BZIP2_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Bz2);
}

#[test]
fn test_valid_tar_bz2_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.tar.bz2");
    create_file_with_magic(&file_path, BZIP2_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarBz2);
}

#[test]
fn test_valid_tbz_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.tbz");
    create_file_with_magic(&file_path, BZIP2_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarBz2);
}

#[test]
fn test_valid_tbz2_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.tbz2");
    create_file_with_magic(&file_path, BZIP2_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarBz2);
}

#[test]
fn test_valid_tar_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.tar");
    create_tar_file_with_magic(&file_path).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Tar);
}

#[test]
fn test_valid_7z_archive() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.7z");
    create_file_with_magic(&file_path, SEVENZ_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::SevenZ);
}

// ============================================================================
// Tests for invalid archives with format spoofing
// ============================================================================

#[test]
fn test_zip_extension_with_gzip_magic() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("fake.zip");
    create_file_with_magic(&file_path, GZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_tar_gz_extension_with_zip_magic() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("fake.tar.gz");
    create_file_with_magic(&file_path, ZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_tar_xz_extension_with_bzip2_magic() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("fake.tar.xz");
    create_file_with_magic(&file_path, BZIP2_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_7z_extension_with_xz_magic() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("fake.7z");
    create_file_with_magic(&file_path, XZ_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_bz2_extension_with_7z_magic() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("fake.bz2");
    create_file_with_magic(&file_path, SEVENZ_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

// ============================================================================
// Tests for unsupported extensions
// ============================================================================

#[test]
fn test_unsupported_extension_txt() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    create_file_with_magic(&file_path, ZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_unsupported_extension_rar() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rar");
    create_file_with_magic(&file_path, ZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_no_extension() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("noextension");
    create_file_with_magic(&file_path, ZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

// ============================================================================
// Tests for corrupted or invalid files
// ============================================================================

#[test]
fn test_empty_file_with_valid_extension() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("empty.zip");
    File::create(&file_path).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_file_too_small_for_magic_bytes() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("tiny.zip");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(&[0x50]).unwrap(); // Only 1 byte

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_zip_file_too_small_for_magic_bytes() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("tiny.zip");
    let mut file = File::create(&file_path).unwrap();
    file.write_all(&[0x50, 0x4B, 0x03]).unwrap(); // Only 3 bytes (zip needs 4)

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_tar_file_too_small_for_offset_magic() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("tiny.tar");
    let mut file = File::create(&file_path).unwrap();
    // Write less than TAR_MAGIC_OFFSET bytes
    file.write_all(&vec![0u8; 100]).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_tar_file_with_wrong_magic_at_offset() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("corrupted.tar");
    let mut file = File::create(&file_path).unwrap();
    // Write TAR_MAGIC_OFFSET bytes, then wrong magic
    file.write_all(&vec![0u8; TAR_MAGIC_OFFSET]).unwrap();
    file.write_all(b"wrong").unwrap();
    file.write_all(&vec![0u8; 256]).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_file_with_random_bytes() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("random.zip");
    create_invalid_file(&file_path).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

// ============================================================================
// Tests for non-existent files
// ============================================================================

#[test]
fn test_nonexistent_file() {
    let file_path = PathBuf::from("/nonexistent/path/file.zip");

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_nonexistent_file_in_temp_dir() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("doesnotexist.tar.gz");

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

// ============================================================================
// Tests for case sensitivity
// ============================================================================

#[test]
fn test_uppercase_extension_zip() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.ZIP");
    create_file_with_magic(&file_path, ZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Zip);
}

#[test]
fn test_mixed_case_extension_tar_gz() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.TAR.GZ");
    create_file_with_magic(&file_path, GZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarGz);
}

#[test]
fn test_mixed_case_tgz() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.TgZ");
    create_file_with_magic(&file_path, GZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarGz);
}

// ============================================================================
// Tests for special filename scenarios
// ============================================================================

#[test]
fn test_filename_with_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("my archive file.zip");
    create_file_with_magic(&file_path, ZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Zip);
}

#[test]
fn test_filename_with_special_characters() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test-file_v1.0.tar.gz");
    create_file_with_magic(&file_path, GZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarGz);
}

#[test]
fn test_filename_with_multiple_dots() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.backup.v1.0.zip");
    create_file_with_magic(&file_path, ZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Zip);
}

#[test]
fn test_very_short_filename() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("a.gz");
    create_file_with_magic(&file_path, GZIP_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Gz);
}

#[test]
fn test_very_long_filename() {
    let temp_dir = TempDir::new().unwrap();
    let long_name = "a".repeat(200);
    let file_path = temp_dir.path().join(format!("{}.tar.xz", long_name));
    create_file_with_magic(&file_path, XZ_MAGIC).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::TarXz);
}

// ============================================================================
// Tests for edge cases with magic bytes
// ============================================================================

#[test]
fn test_partial_zip_magic_bytes() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("partial.zip");
    let mut file = File::create(&file_path).unwrap();
    // Write partial zip magic (PK but not the rest)
    file.write_all(&[0x50, 0x4B, 0x00, 0x00]).unwrap();
    file.write_all(&vec![0u8; 512]).unwrap();

    let format = get_validated_archive_format(&file_path);
    assert!(format.is_err());
}

#[test]
fn test_gzip_magic_with_extra_bytes() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.gz");
    let mut file = File::create(&file_path).unwrap();
    // Write correct gzip magic
    file.write_all(GZIP_MAGIC).unwrap();
    // Write some valid-looking gzip header data
    file.write_all(&[0x08, 0x00, 0x00, 0x00, 0x00, 0x00])
        .unwrap();
    file.write_all(&vec![0u8; 512]).unwrap();

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Gz);
}

#[test]
fn test_xz_magic_full_sequence() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.xz");
    create_file_with_magic(&file_path, XZ_MAGIC).unwrap();

    // Verify XZ magic is 6 bytes
    assert_eq!(XZ_MAGIC.len(), 6);

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Xz);
}

#[test]
fn test_sevenz_magic_full_sequence() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.7z");
    create_file_with_magic(&file_path, SEVENZ_MAGIC).unwrap();

    // Verify 7z magic is 6 bytes
    assert_eq!(SEVENZ_MAGIC.len(), 6);

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::SevenZ);
}

#[test]
fn test_tar_format_requires_special_handling() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.tar");
    create_tar_file_with_magic(&file_path).unwrap();

    // Verify tar magic is at offset 257, not at beginning
    let mut file = File::open(&file_path).unwrap();
    let mut buffer = vec![0u8; 512];
    std::io::Read::read_exact(&mut file, &mut buffer).unwrap();

    // First bytes should be zeros, not tar magic
    assert_ne!(&buffer[0..5], TAR_MAGIC);
    // Tar magic should be at offset 257
    assert_eq!(&buffer[TAR_MAGIC_OFFSET..TAR_MAGIC_OFFSET + 5], TAR_MAGIC);

    let format = get_validated_archive_format(&file_path).unwrap();
    assert_eq!(format, BinaryContainer::Tar);
}

// ============================================================================
// Tests for security scenarios (format spoofing)
// ============================================================================

#[test]
fn test_executable_with_archive_extension() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("virus.zip");
    // Put ELF magic bytes (executable)
    let elf_magic = &[0x7F, 0x45, 0x4C, 0x46];
    create_file_with_magic(&file_path, elf_magic).unwrap();

    let format = get_validated_archive_format(&file_path);
    // Should detect this is not a zip and return Unknown
    assert!(format.is_err());
}
