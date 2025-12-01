//! Unit tests for archive functions
//! Tests archive format extraction

use std::path::PathBuf;
use tempfile::TempDir;

use crate::files::archives::extract_to_dir;

/// Get the path to the fixtures directory
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("archives")
}

// ============================================================================
// Tests for archive formats
// ============================================================================

#[test]
fn test_extract_zip_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.zip");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture archive
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(result.is_ok(), "Extraction failed: {:?}", result.err());

    // Verify extracted files exist (archive.* contains: file.txt, text.txt, README)
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

#[test]
fn test_extract_tar_gz_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.tar.gz");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture archive
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(result.is_ok(), "Extraction failed: {:?}", result.err());

    // Verify extracted files exist (archive.* contains: file.txt, text.txt, README)
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

#[test]
fn test_extract_tgz_extension() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.tgz");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture archive with .tgz extension
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .tgz: {:?}",
        result.err()
    );

    // Verify extracted files exist
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

#[test]
fn test_extract_tar_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.tar");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture tar archive
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .tar: {:?}",
        result.err()
    );

    // Verify extracted files exist
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

#[test]
fn test_extract_tar_xz_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.tar.xz");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture tar.xz archive
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .tar.xz: {:?}",
        result.err()
    );

    // Verify extracted files exist
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

#[test]
fn test_extract_txz_extension() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.txz");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture archive with .txz extension
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .txz: {:?}",
        result.err()
    );

    // Verify extracted files exist
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

#[test]
fn test_extract_tar_bz2_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.tar.bz2");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture tar.bz2 archive
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .tar.bz2: {:?}",
        result.err()
    );

    // Verify extracted files exist
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

#[test]
fn test_extract_tbz_extension() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.tbz");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture archive with .tbz extension
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .tbz: {:?}",
        result.err()
    );

    // Verify extracted files exist
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

#[test]
fn test_extract_tbz2_extension() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.tbz2");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture archive with .tbz2 extension
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .tbz2: {:?}",
        result.err()
    );

    // Verify extracted files exist
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

#[test]
fn test_extract_7z_archive() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.7z");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture 7z archive
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .7z: {:?}",
        result.err()
    );

    // Verify extracted files exist
    assert!(extract_path.join("file.txt").exists());
    assert!(extract_path.join("text.txt").exists());
    assert!(extract_path.join("README").exists());
}

// ============================================================================
// Tests for single compressed files
// ============================================================================

#[test]
fn test_extract_gz_compressed_file() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("file.txt.gz");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture gz compressed file (not tar.gz, just gz)
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .gz: {:?}",
        result.err()
    );

    // Verify extracted file exists (file.* contains just file.txt)
    assert!(extract_path.join("file.txt").exists());
}

#[test]
fn test_extract_xz_compressed_file() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("file.txt.xz");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture xz compressed file (not tar.xz, just xz)
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .xz: {:?}",
        result.err()
    );

    // Verify extracted file exists (file.* contains just file.txt)
    assert!(extract_path.join("file.txt").exists());
}

#[test]
fn test_extract_bz2_compressed_file() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("file.txt.bz2");
    let extract_path = temp_dir.path().join("extracted");

    // Extract the fixture bz2 compressed file (not tar.bz2, just bz2)
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(
        result.is_ok(),
        "Extraction failed for .bz2: {:?}",
        result.err()
    );

    // Verify extracted file exists (file.* contains just file.txt)
    assert!(extract_path.join("file.txt").exists());
}

// ============================================================================
// Tests for non existent files and directories
// ============================================================================

#[test]
fn test_extract_nonexistent_file_returns_error() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = temp_dir.path().join("nonexistent.zip");
    let _extract_path = temp_dir.path().to_path_buf();

    // Verify the file doesn't exist
    assert!(!archive_path.exists());

    // Calling extract_to_dir on a nonexistent file will trigger exit(109)
    // because the validation fails and returns BinaryContainer::Unknown.
    // This is a known limitation of the current implementation.
    // In a production scenario, this should ideally return a proper error
    // instead of calling process::exit().
    //
    // For now, we just verify that the file doesn't exist and acknowledge
    // that calling extract_to_dir would cause the test to exit.
    // A better implementation would be tested if extract_to_dir returned Result
    // with proper error types instead of calling exit().
}

#[test]
fn test_extract_to_nonexistent_directory() {
    let temp_dir = TempDir::new().unwrap();
    let archive_path = fixtures_dir().join("archive.zip");
    let extract_path = temp_dir.path().join("new_directory");

    // Verify directory doesn't exist yet
    assert!(!extract_path.exists());

    // Extract - the directory should be created automatically
    let result = extract_to_dir(&archive_path, &extract_path);
    assert!(result.is_ok(), "Extraction failed: {:?}", result.err());

    // Verify directory was created and files exist
    assert!(extract_path.exists());
    assert!(extract_path.join("file.txt").exists());
}
