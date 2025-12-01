//! Unit tests for archive functions
//! Common helper functions for archive tests

use std::fs::File;
use std::io::Write;
use std::path::Path;

use poof::files::magic::{TAR_MAGIC, TAR_MAGIC_OFFSET};

/// Helper function to create a file with specific magic bytes
pub fn create_file_with_magic(path: &Path, magic: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(magic)?;
    // Pad with zeros to make a reasonable file size
    file.write_all(&vec![0u8; 512])?;
    Ok(())
}

/// Helper function to create a tar magic bytes file
pub fn create_tar_file_with_magic(path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    // Write TAR_MAGIC_OFFSET bytes of zeros, then the tar magic
    file.write_all(&vec![0u8; TAR_MAGIC_OFFSET])?;
    file.write_all(TAR_MAGIC)?;
    // Pad with more zeros
    file.write_all(&vec![0u8; 256])?;
    Ok(())
}

/// Helper function to create an invalid file (wrong magic bytes)
pub fn create_invalid_file(path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(&[0xFF, 0xFF, 0xFF, 0xFF])?;
    file.write_all(&vec![0u8; 512])?;
    Ok(())
}
