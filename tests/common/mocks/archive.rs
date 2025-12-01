//! Mocks for archive creation
//! This module provides a helper function to create a mock release archive structure
//! for testing purposes.

use std::path::{Path, PathBuf};

/// Helper to create a mock release archive structure
#[allow(dead_code)]
pub fn create_mock_archive_structure(
    base_dir: &Path,
    binary_name: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let archive_dir = base_dir.join("archive");
    std::fs::create_dir_all(&archive_dir)?;

    let binary_path = archive_dir.join(binary_name);
    std::fs::write(&binary_path, b"#!/bin/sh\necho 'mock binary'")?;

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms)?;
    }

    Ok(archive_dir)
}
