//! Test fixture for setting up a temporary environment for testing
//! This ensures tests never touch the actual file system
//! and provides a clean, isolated environment for testing.

use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test fixture that sets up a temporary environment for testing
/// This ensures tests never touch the actual file system
///
/// **Important**: To avoid race conditions in parallel test execution,
/// use `.env()` on `Command` instances instead of `std::env::set_var()`.
/// The fixture provides paths that should be passed to `.env()` calls.
pub struct TestFixture {
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    pub home_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub bin_dir: PathBuf,
    // Removed: original env vars no longer needed since we don't set them globally
    #[allow(dead_code)]
    original_home: Option<String>,
    #[allow(dead_code)]
    original_xdg_data_home: Option<String>,
    #[allow(dead_code)]
    original_xdg_cache_home: Option<String>,
}

impl TestFixture {
    /// Create a new test fixture with temporary directories
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let home_dir = temp_dir.path().to_path_buf();

        // Create directory structure
        let data_dir = home_dir
            .join(".local")
            .join("share")
            .join("poof")
            .join("data");
        let cache_dir = home_dir.join(".cache").join("poof");
        let bin_dir = home_dir
            .join(".local")
            .join("share")
            .join("poof")
            .join("bin");

        std::fs::create_dir_all(&data_dir)?;
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&bin_dir)?;

        // Note: Environment variables should be set using .env() on Command instances
        // to avoid race conditions when tests run in parallel. We don't set them here.

        Ok(Self {
            temp_dir,
            home_dir,
            data_dir,
            cache_dir,
            bin_dir,
            original_home: None,
            original_xdg_data_home: None,
            original_xdg_cache_home: None,
        })
    }

    /// Create a fake binary installation for testing
    pub fn create_fake_installation(
        &self,
        repo: &str,
        version: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let separator = std::path::MAIN_SEPARATOR.to_string();
        let install_dir = self
            .data_dir
            .join(repo.replace('/', &separator))
            .join(version);

        std::fs::create_dir_all(&install_dir)?;

        // Create a fake executable
        let binary_name = repo.split('/').next_back().unwrap_or("binary");
        let binary_path = install_dir.join(binary_name);
        std::fs::write(&binary_path, b"#!/bin/sh\necho 'fake binary'")?;

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&binary_path, perms)?;
        }

        Ok(install_dir)
    }

    /// Create a symlink in bin_dir pointing to the installed binary
    #[allow(dead_code)]
    pub fn create_bin_symlink(
        &self,
        binary_name: &str,
        target: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let symlink_path = self.bin_dir.join(binary_name);

        #[cfg(not(target_os = "windows"))]
        {
            if symlink_path.exists() {
                std::fs::remove_file(&symlink_path)?;
            }
            std::os::unix::fs::symlink(target, &symlink_path)?;
        }

        #[cfg(target_os = "windows")]
        {
            // On Windows, we'd use junctions or copy, but for tests we'll skip
            // since the codebase targets Unix-like systems
        }

        Ok(())
    }

    /// Get the path to a specific binary installation
    pub fn get_install_path(&self, repo: &str, version: &str) -> PathBuf {
        let separator = std::path::MAIN_SEPARATOR.to_string();
        self.data_dir
            .join(repo.replace('/', &separator))
            .join(version)
    }

    /// Check if a binary is installed
    pub fn is_binary_installed(&self, repo: &str, version: &str) -> bool {
        let install_dir = self.get_install_path(repo, version);
        install_dir.exists() && install_dir.is_dir()
    }

    /// List all installed binaries
    #[allow(dead_code)]
    pub fn list_installed(&self) -> Vec<(String, String)> {
        let mut result = Vec::new();

        if !self.data_dir.exists() {
            return result;
        }

        if let Ok(entries) = std::fs::read_dir(&self.data_dir) {
            for user_entry in entries.flatten() {
                if !user_entry.path().is_dir() {
                    continue;
                }

                let username = user_entry.file_name().to_string_lossy().to_string();

                if let Ok(repo_entries) = std::fs::read_dir(user_entry.path()) {
                    for repo_entry in repo_entries.flatten() {
                        if !repo_entry.path().is_dir() {
                            continue;
                        }

                        let repo_name = repo_entry.file_name().to_string_lossy().to_string();
                        let slug = format!("{}/{}", username, repo_name);

                        if let Ok(version_entries) = std::fs::read_dir(repo_entry.path()) {
                            for version_entry in version_entries.flatten() {
                                if version_entry.path().is_dir() {
                                    let version =
                                        version_entry.file_name().to_string_lossy().to_string();
                                    result.push((slug.clone(), version));
                                }
                            }
                        }
                    }
                }
            }
        }

        result
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // No environment variable cleanup needed since we don't set them globally
        // Tests should use .env() on Command instances for proper isolation
    }
}
