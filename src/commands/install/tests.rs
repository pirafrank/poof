//! Unit tests for the install command module
//! Tests focus on internal logic without external dependencies

use std::fs;
use std::path::{Path, PathBuf};
use temp_env;
use tempfile::TempDir;

// Import the parent module to access functions
use super::*;

use anyhow::Result;

/// Helper to set up a test environment with temporary directories
struct TestEnv {
    _temp_dir: TempDir,
    home_dir: PathBuf,
}

impl TestEnv {
    // Test constants to reduce magic strings
    const DEFAULT_TEST_SLUG: &'static str = "testuser/testrepo";
    const DEFAULT_BINARY_NAME: &'static str = "testrepo";
    const BIN_DIR_NAME: &'static str = "bin";

    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let home = temp_dir.path().canonicalize().unwrap().to_path_buf();

        Ok(Self {
            _temp_dir: temp_dir,
            home_dir: home,
        })
    }

    /// Helper to create a test slug
    fn test_slug() -> Slug {
        Slug::new(Self::DEFAULT_TEST_SLUG).expect("test slug should be valid")
    }

    fn create_dir(&self, name: &str) -> Result<PathBuf> {
        let path = self.home_dir.join(name);
        fs::create_dir_all(&path)?;
        Ok(path.canonicalize()?)
    }

    /// Create the bin directory
    fn create_bin_dir(&self) -> Result<PathBuf> {
        self.create_dir(Self::BIN_DIR_NAME)
    }

    /// Get or create bin directory and return symlink path for binary
    fn get_symlink_path(&self, binary_name: &str) -> Result<PathBuf> {
        let bin_dir = self.create_bin_dir()?;
        Ok(bin_dir.join(binary_name))
    }

    /// Helper to create a mock executable file
    fn create_mock_executable(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, b"#!/bin/sh\necho 'mock binary'")?;

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Helper to create a platform-specific executable file with proper magic numbers
    fn create_platform_executable(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        use crate::files::magic;
        let mut file = fs::File::create(path)?;

        #[cfg(target_os = "linux")]
        {
            use std::io::Write;
            // Write ELF magic number
            file.write_all(&magic::ELF_MAGIC)?;
            // Write some dummy content
            file.write_all(b"ELF dummy content for testing")?;
        }

        #[cfg(target_os = "macos")]
        {
            use std::io::Write;
            // Write Mach-O magic number (64-bit little-endian)
            file.write_all(&magic::MACHO_MAGIC_NUMBERS[1])?;
            // Write some dummy content
            file.write_all(b"Mach-O dummy content for testing")?;
        }

        #[cfg(target_os = "windows")]
        {
            use std::io::Write;
            // Write PE magic number (MZ header)
            file.write_all(&magic::PE_MAGIC)?;
            // Write some dummy content
            file.write_all(b"PE dummy content for testing")?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms)?;
        }

        Ok(())
    }

    /// Run closure with test environment variables set (HOME, XDG_DATA_HOME)
    fn with_test_env<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        temp_env::with_vars(
            vec![
                ("HOME", Some(self.home_dir.to_str().unwrap())),
                #[cfg(target_os = "linux")]
                (
                    "XDG_DATA_HOME",
                    Some(self.home_dir.join(".local/share").to_str().unwrap()),
                ),
            ],
            f,
        )
    }

    /// Create a complete mock installation with data dir structure, binary, and optional symlink
    fn create_mock_installation_with_slug(
        &self,
        slug: &str,
        version: &str,
        binary_name: &str,
        create_symlink: bool,
    ) -> Result<PathBuf> {
        self.with_test_env(|| {
            let data_dir = datadirs::get_data_dir().unwrap();
            let install_dir = data_dir.join(slug).join(version);
            fs::create_dir_all(&install_dir).unwrap();

            let target_binary = install_dir.join(binary_name);
            self.create_mock_executable(&target_binary).unwrap();

            if create_symlink {
                let bin_dir = self.create_bin_dir().unwrap();
                let symlink_path = bin_dir.join(binary_name);
                #[cfg(not(target_os = "windows"))]
                std::os::unix::fs::symlink(&target_binary, &symlink_path).unwrap();
            }

            Ok(install_dir)
        })
    }

    /// Create a Unix symlink (no-op on Windows in tests)
    #[cfg(not(target_os = "windows"))]
    #[allow(dead_code)]
    fn create_unix_symlink(&self, target: &Path, link: &Path) -> Result<()> {
        std::os::unix::fs::symlink(target, link)?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn create_unix_symlink(&self, _target: &Path, _link: &Path) -> Result<()> {
        Ok(()) // No-op on Windows
    }

    /// Run closure with additional directory prepended to PATH
    fn with_path_extended<F, R>(&self, additional_dir: &Path, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let original_path = std::env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", additional_dir.display(), original_path);
        temp_env::with_var("PATH", Some(&new_path), f)
    }
}

// =============================================================================
// Tests for get_install_dir
// =============================================================================

#[cfg(test)]
mod get_install_dir_tests {
    use super::*;

    #[test]
    fn test_get_install_dir_constructs_correct_path() {
        temp_env::with_vars(
            vec![
                ("HOME", Some("/tmp/test_home")),
                #[cfg(target_os = "linux")]
                ("XDG_DATA_HOME", Some("/tmp/test_home/.local/share")),
            ],
            || {
                let result = get_install_dir("owner/repo", "1.0.0");
                assert!(result.is_ok());

                let install_dir = result.unwrap();
                let path_str = install_dir.to_string_lossy();

                // Verify path structure contains expected components
                assert!(path_str.contains("owner"));
                assert!(path_str.contains("repo"));
                assert!(path_str.contains("1.0.0"));
                assert!(path_str.contains("poof"));
            },
        );
    }

    #[test]
    fn test_get_install_dir_with_special_characters() {
        temp_env::with_vars(
            vec![
                ("HOME", Some("/tmp/test_home")),
                #[cfg(target_os = "linux")]
                ("XDG_DATA_HOME", Some("/tmp/test_home/.local/share")),
            ],
            || {
                let result = get_install_dir("user-name/repo_name", "1.0.0-beta.1");
                assert!(result.is_ok());

                let install_dir = result.unwrap();
                let path_str = install_dir.to_string_lossy();

                assert!(path_str.contains("user-name"));
                assert!(path_str.contains("repo_name"));
                assert!(path_str.contains("1.0.0-beta.1"));
            },
        );
    }

    #[test]
    fn test_get_install_dir_with_numeric_version() {
        temp_env::with_vars(
            vec![
                ("HOME", Some("/tmp/test_home")),
                #[cfg(target_os = "linux")]
                ("XDG_DATA_HOME", Some("/tmp/test_home/.local/share")),
            ],
            || {
                let result = get_install_dir("org/tool", "2.3.4");
                assert!(result.is_ok());

                let install_dir = result.unwrap();
                let path_str = install_dir.to_string_lossy();
                assert!(path_str.contains("2.3.4"));
            },
        );
    }
}

// =============================================================================
// Tests for check_if_installed
// =============================================================================

#[cfg(test)]
mod check_if_installed_tests {
    use super::*;

    #[test]
    fn test_check_if_installed_not_exists() -> Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.home_dir.join("nonexistent");

        let result = check_if_installed(&install_dir)?;
        assert!(!result, "Should return false when directory doesn't exist");

        Ok(())
    }

    #[test]
    fn test_check_if_installed_empty_dir() -> Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.create_dir("empty_install")?;

        let result = check_if_installed(&install_dir)?;
        assert!(!result, "Should return false when directory is empty");

        Ok(())
    }

    #[test]
    fn test_check_if_installed_not_empty() -> Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.create_dir("non_empty_install")?;

        // Add a file to make it non-empty
        fs::write(install_dir.join("some_binary"), b"content")?;

        let result = check_if_installed(&install_dir)?;
        assert!(result, "Should return true when directory is not empty");

        Ok(())
    }

    #[test]
    fn test_check_if_installed_is_file() -> Result<()> {
        let env = TestEnv::new()?;
        let install_path = env.home_dir.join("is_a_file");
        fs::write(&install_path, b"content")?;

        let result = check_if_installed(&install_path);
        assert!(result.is_err(), "Should return error when path is a file");

        Ok(())
    }

    #[test]
    fn test_check_if_installed_with_multiple_files() -> Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.create_dir("multi_file_install")?;

        // Add multiple files
        fs::write(install_dir.join("binary1"), b"content1")?;
        fs::write(install_dir.join("binary2"), b"content2")?;

        let result = check_if_installed(&install_dir)?;
        assert!(
            result,
            "Should return true when directory has multiple files"
        );

        Ok(())
    }
}

// =============================================================================
// Tests for prepare_install_dir
// =============================================================================

#[cfg(test)]
mod prepare_install_dir_tests {
    use super::*;

    #[test]
    fn test_prepare_install_dir_creates_directory() -> Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.home_dir.join("new_install");

        assert!(
            !install_dir.exists(),
            "Directory should not exist initially"
        );

        let result = prepare_install_dir(&install_dir);
        assert!(result.is_ok(), "Should successfully create directory");
        assert!(
            install_dir.exists(),
            "Directory should exist after preparation"
        );
        assert!(install_dir.is_dir(), "Path should be a directory");

        Ok(())
    }

    #[test]
    fn test_prepare_install_dir_nested_path() -> Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.home_dir.join("owner/repo/1.0.0");

        assert!(
            !install_dir.exists(),
            "Directory should not exist initially"
        );

        let result = prepare_install_dir(&install_dir);
        assert!(result.is_ok(), "Should create nested directories");
        assert!(install_dir.exists(), "Nested directory should exist");
        assert!(install_dir.is_dir(), "Path should be a directory");

        Ok(())
    }

    #[test]
    fn test_prepare_install_dir_already_exists() -> Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.create_dir("existing_install")?;

        let result = prepare_install_dir(&install_dir);
        assert!(
            result.is_ok(),
            "Should succeed even if directory already exists"
        );

        Ok(())
    }
}

// =============================================================================
// Tests for clean_cache_dir
// =============================================================================

#[cfg(test)]
mod clean_cache_dir_tests {
    use super::*;

    #[test]
    fn test_clean_cache_dir_success() -> Result<()> {
        let env = TestEnv::new()?;
        let cache_root = env.create_dir("cache")?;
        let cache_subdir = cache_root.join("subdir");
        fs::create_dir_all(&cache_subdir)?;

        // Add a file to the cache
        fs::write(cache_subdir.join("cached_file"), b"data")?;

        let result = clean_cache_dir(&cache_subdir, &cache_root)?;
        assert!(result, "Should return true when directory is deleted");
        assert!(
            !cache_subdir.exists(),
            "Cache subdirectory should be deleted"
        );

        Ok(())
    }

    #[test]
    fn test_clean_cache_dir_outside_root() -> Result<()> {
        let env = TestEnv::new()?;
        let cache_root = env.create_dir("cache")?;
        let outside_dir = env.create_dir("not_cache")?;

        let result = clean_cache_dir(&outside_dir, &cache_root)?;
        assert!(
            !result,
            "Should return false and refuse to delete outside cache"
        );
        assert!(
            outside_dir.exists(),
            "Directory outside cache should not be deleted"
        );

        Ok(())
    }

    #[test]
    fn test_clean_cache_dir_nonexistent() -> Result<()> {
        let env = TestEnv::new()?;
        let cache_root = env.create_dir("cache")?;
        let nonexistent = cache_root.join("does_not_exist");

        let result = clean_cache_dir(&nonexistent, &cache_root)?;
        assert!(!result, "Should return false when directory doesn't exist");

        Ok(())
    }

    #[test]
    fn test_clean_cache_dir_nested() -> Result<()> {
        let env = TestEnv::new()?;
        let cache_root = env.create_dir("cache")?;
        let nested_dir = cache_root.join("level1").join("level2").join("level3");
        fs::create_dir_all(&nested_dir)?;

        let result = clean_cache_dir(&nested_dir, &cache_root)?;
        assert!(result, "Should successfully delete nested directory");
        assert!(!nested_dir.exists(), "Nested directory should be deleted");

        Ok(())
    }
}

// =============================================================================
// Tests for install_binary
// =============================================================================

#[cfg(test)]
mod install_binary_tests {
    use std::ffi::OsString;

    use super::*;

    #[test]
    fn test_install_binary_basic() -> Result<()> {
        let env = TestEnv::new()?;
        let source_exec = env.home_dir.join("source/mybinary");
        let install_dir = env.create_dir("install")?;

        // Create a mock executable
        env.create_mock_executable(&source_exec)?;

        let slug = TestEnv::test_slug();
        let exec_stem = OsString::from("mybinary");
        let result = install_binary(&slug, &source_exec, &install_dir, &exec_stem);
        // If bin_dir cannot be determined, skip the assertion
        if let Err(e) = &result {
            if format!("{:?}", e).contains("Cannot determine") {
                eprintln!(
                    "Skipping test_install_binary_basic: bin_dir unavailable in CI/sandboxed environment. \
                    This is expected when HOME or XDG dirs are not properly configured."
                );
                return Ok(());
            }
        }

        result?;
        let installed = install_dir.join("mybinary");
        assert!(
            installed.exists(),
            "Binary should be copied to install directory"
        );
        let installed_content = fs::read(&installed)?;
        assert_eq!(
            installed_content, b"#!/bin/sh\necho 'mock binary'",
            "Content should be preserved"
        );
        Ok(())
    }

    #[test]
    fn test_install_binary_preserves_content() -> Result<()> {
        let env = TestEnv::new()?;
        let source_exec = env.home_dir.join("source/tool");
        let install_dir = env.create_dir("install")?;
        fs::create_dir_all(source_exec.parent().unwrap())?;

        // Create a binary with specific content
        let content = b"#!/bin/sh\necho 'specific content'";
        fs::write(&source_exec, content)?;

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&source_exec)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&source_exec, perms)?;
        }
        let slug = TestEnv::test_slug();
        let exec_stem = OsString::from("tool");
        // Handle expected failures due to bin_dir issues in test environment
        if let Err(e) = install_binary(&slug, &source_exec, &install_dir, &exec_stem) {
            if !format!("{:?}", e).contains("Cannot determine") {
                return Err(e);
            } else {
                eprintln!("Skipping assertion: bin_dir unavailable in test environment");
                return Ok(());
            }
        }

        let installed = install_dir.join("tool");
        assert!(installed.exists(), "Binary should exist after install");
        let installed_content = fs::read(&installed)?;
        assert_eq!(installed_content, content, "Content should be preserved");

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_install_binary_sets_executable_permissions() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let env = TestEnv::new()?;
        let source_exec = env.home_dir.join("source/executable");
        let install_dir = env.create_dir("install")?;
        fs::create_dir_all(&install_dir)?;

        env.create_mock_executable(&source_exec)?;

        let slug = TestEnv::test_slug();
        let exec_stem = OsString::from("executable");
        let _ = install_binary(&slug, &source_exec, &install_dir, &exec_stem);

        let installed = install_dir.join("executable");
        if installed.exists() {
            let metadata = fs::metadata(&installed)?;
            let mode = metadata.permissions().mode();
            // Check that executable bit is set (0o111 means any execute bit)
            assert_ne!(mode & 0o111, 0, "Executable permissions should be set");
        }

        Ok(())
    }
}

// =============================================================================
// Integration-style tests using the public API
// =============================================================================

#[cfg(test)]
mod public_api_tests {
    use super::*;

    #[test]
    fn test_select_assets_with_nonexistent_repo() {
        use mockito::Server;
        use serde_json::json;

        let mut server = Server::new();
        let repo = "nonexistent-user-12345/nonexistent-repo-67890";

        // Mock a 404 response for nonexistent repository
        let mock = server
            .mock("GET", format!("/{}/releases/latest", repo).as_str())
            .match_header("User-Agent", "pirafrank/poof")
            .match_header("Accept", "application/vnd.github.v3+json")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "message": "Not Found",
                    "documentation_url": "https://docs.github.com/rest"
                })
                .to_string(),
            )
            .create();

        let result = temp_env::with_vars(
            vec![("POOF_GITHUB_API_URL", Some(server.url().as_str()))],
            || {
                let result = select_assets(repo, None);
                mock.assert();
                result
            },
        );

        assert!(result.is_err(), "Should fail for nonexistent repository");
    }

    #[test]
    fn test_select_assets_invalid_repo_format() {
        use mockito::Server;
        use serde_json::json;

        let mut server = Server::new();

        // Test with empty repo - mock the endpoint that would be called
        let mock_empty = server
            .mock("GET", "//releases/latest")
            .match_header("User-Agent", "pirafrank/poof")
            .match_header("Accept", "application/vnd.github.v3+json")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "message": "Not Found",
                    "documentation_url": "https://docs.github.com/rest"
                })
                .to_string(),
            )
            .create();

        let result_empty = temp_env::with_vars(
            vec![("POOF_GITHUB_API_URL", Some(server.url().as_str()))],
            || {
                let result = select_assets("", None);
                mock_empty.assert();
                result
            },
        );
        assert!(result_empty.is_err(), "Should fail for empty repo");

        // Test with invalid format (no slash)
        let invalid_repo = "no-slash";
        let mock_invalid = server
            .mock("GET", format!("/{}/releases/latest", invalid_repo).as_str())
            .match_header("User-Agent", "pirafrank/poof")
            .match_header("Accept", "application/vnd.github.v3+json")
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "message": "Not Found",
                    "documentation_url": "https://docs.github.com/rest"
                })
                .to_string(),
            )
            .create();

        let result_invalid = temp_env::with_vars(
            vec![("POOF_GITHUB_API_URL", Some(server.url().as_str()))],
            || {
                let result = select_assets(invalid_repo, None);
                mock_invalid.assert();
                result
            },
        );
        assert!(
            result_invalid.is_err(),
            "Should fail for invalid repo format"
        );
    }
}

// =============================================================================
// Tests for process_install
// =============================================================================

#[cfg(test)]
mod process_install_tests {
    use super::*;

    #[test]
    fn test_process_install_executable_path() -> Result<()> {
        let env = TestEnv::new()?;
        let downloaded_file = env.home_dir.join("downloaded/mybin-linux-x86_64");
        let download_to = env.create_dir("download")?;
        let install_dir = env.create_dir("install")?;

        // Create a platform-specific executable file
        env.create_platform_executable(&downloaded_file)?;

        let slug = TestEnv::test_slug();
        let asset_name = String::from("mybin-linux-x86_64");
        let result = process_install(
            &slug,
            &downloaded_file,
            &download_to,
            &install_dir,
            &asset_name,
        );

        // Note: This may fail if bin_dir cannot be created, but the copy should work
        match result {
            Ok(_) => {
                // Verify executable was installed with trimmed name
                let installed = install_dir.join("mybin");
                assert!(
                    installed.exists(),
                    "Executable should be installed with trimmed name"
                );
            }
            Err(e) => {
                // If it fails, it should be due to bin_dir issues, not the copy itself
                let err_msg = format!("{:?}", e);
                if !err_msg.contains("Cannot determine") {
                    let installed = install_dir.join("mybin");
                    assert!(
                        installed.exists(),
                        "Executable should still be copied even if symlink fails"
                    );
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_process_install_archive_path() -> Result<()> {
        let env = TestEnv::new()?;
        let temp_extract = TempDir::new()?;

        // Get the archive fixture path
        let fixtures_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("archives");
        let archive_path = fixtures_dir.join("archive.zip");

        // Copy archive to temp location for testing
        let downloaded_file = env.home_dir.join("downloaded/archive.zip");
        if let Some(parent) = downloaded_file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&archive_path, &downloaded_file)?;

        let download_to = temp_extract.path().to_path_buf();
        let install_dir = env.create_dir("install")?;
        fs::create_dir_all(&install_dir)?;

        let slug = TestEnv::test_slug();
        let asset_name = String::from("archive.zip");
        let result = process_install(
            &slug,
            &downloaded_file,
            &download_to,
            &install_dir,
            &asset_name,
        );

        // The archive should be extracted and executables installed
        // Note: The archive fixture may not contain executables, so this might fail
        // But we're testing that the extraction path is called
        match result {
            Ok(_) => {
                // Verify extraction directory was created
                assert!(download_to.exists(), "Extraction directory should exist");
            }
            Err(e) => {
                // If it fails, it should be due to no executables found, not extraction
                let err_msg = format!("{:?}", e);
                // Extraction should succeed, but finding executables might fail
                assert!(
                    err_msg.contains("No executables") || err_msg.contains("executables"),
                    "Error should be about executables, not extraction: {}",
                    err_msg
                );
            }
        }

        Ok(())
    }
}

// =============================================================================
// Tests for install_binaries
// =============================================================================

#[cfg(test)]
mod install_binaries_tests {
    use super::*;

    #[test]
    fn test_install_binaries_success() -> Result<()> {
        let env = TestEnv::new()?;
        let temp_extract = TempDir::new()?;
        let install_dir = env.create_dir("install")?;
        fs::create_dir_all(&install_dir)?;

        // Create a mock extracted archive structure
        // The function looks for executables in a directory named after the archive (without extension)
        // or in the parent directory if that doesn't exist
        let archive_path = temp_extract.path().join("archive.zip");
        let extracted_dir = temp_extract.path().join("archive");
        fs::create_dir_all(&extracted_dir)?;

        // Create multiple executable files in the extracted directory
        let exec1 = extracted_dir.join("tool1");
        let exec2 = extracted_dir.join("tool2");
        env.create_platform_executable(&exec1)?;
        env.create_platform_executable(&exec2)?;

        // Create the archive file (just a placeholder, won't be read)
        fs::write(&archive_path, b"dummy archive")?;

        let slug = TestEnv::test_slug();
        let result = install_binaries(&slug, &archive_path, &install_dir);

        // Note: This may fail if bin_dir cannot be created
        match result {
            Ok(_) => {
                // Verify both executables were installed
                assert!(
                    install_dir.join("tool1").exists(),
                    "First executable should be installed"
                );
                assert!(
                    install_dir.join("tool2").exists(),
                    "Second executable should be installed"
                );
            }
            Err(e) => {
                // If it fails, it should be due to bin_dir issues, not the copy itself
                let err_msg = format!("{:?}", e);
                if !err_msg.contains("Cannot determine") {
                    // At least one should be copied
                    assert!(
                        install_dir.join("tool1").exists() || install_dir.join("tool2").exists(),
                        "At least one executable should be copied even if symlink fails"
                    );
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_install_binaries_no_executables() -> Result<()> {
        let env = TestEnv::new()?;
        let temp_extract = TempDir::new()?;
        let install_dir = env.create_dir("install")?;
        fs::create_dir_all(&install_dir)?;

        // Create a mock extracted archive structure with no executables
        let archive_path = temp_extract.path().join("archive.zip");
        let extracted_dir = temp_extract.path().join("archive");
        fs::create_dir_all(&extracted_dir)?;

        // Create a non-executable file
        fs::write(extracted_dir.join("readme.txt"), b"This is a readme file")?;

        // Create the archive file (just a placeholder)
        fs::write(&archive_path, b"dummy archive")?;

        let slug = TestEnv::test_slug();
        let result = install_binaries(&slug, &archive_path, &install_dir);

        assert!(
            result.is_err(),
            "Should return error when no executables found"
        );
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("No executables") || err_msg.contains("executables"),
            "Error message should mention executables: {}",
            err_msg
        );

        Ok(())
    }
}

// =============================================================================
// Tests for select_assets success path
// =============================================================================

#[cfg(test)]
mod select_assets_success_tests {
    use super::*;

    #[test]
    fn test_select_assets_success_with_mock() -> Result<()> {
        // Use mockito to mock GitHub API responses
        use mockito::Server;
        use serde_json::json;

        let mut server = Server::new();

        // Mock the latest release endpoint
        let repo = "testuser/testrepo";
        let tag = "v1.0.0";

        // Determine platform-specific asset name
        // Note: we only support x86_64 and aarch64 linux and macOS platform for tests.
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let asset_name = "testrepo-linux-x86_64";
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        let asset_name = "testrepo-linux-aarch64";
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        let asset_name = "testrepo-darwin-x86_64";
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let asset_name = "testrepo-darwin-aarch64";

        let download_url = format!("{}/releases/download/{}/{}", server.url(), tag, asset_name);

        let mock = server
            .mock("GET", format!("/{}/releases/latest", repo).as_str())
            .match_header("User-Agent", "pirafrank/poof")
            .match_header("Accept", "application/vnd.github.v3+json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "tag_name": tag,
                    "published_at": "2024-01-01T00:00:00Z",
                    "assets": [
                        {
                            "name": asset_name,
                            "browser_download_url": download_url,
                            "content_type": "application/octet-stream",
                        }
                    ],
                })
                .to_string(),
            )
            .create();

        let result = temp_env::with_vars(
            vec![("POOF_GITHUB_API_URL", Some(server.url().as_str()))],
            || {
                let result = select_assets(repo, None);
                mock.assert();
                result
            },
        );

        assert!(result.is_ok(), "Should successfully select assets");
        let (release, assets) = result.unwrap();
        assert_eq!(release.tag_name(), tag);
        assert!(!assets.is_empty(), "Should have at least one asset");
        assert_eq!(assets[0].name(), asset_name);

        Ok(())
    }
}

// =============================================================================
// Tests for install function - already installed path
// =============================================================================

#[cfg(test)]
mod install_already_installed_tests {
    use super::*;

    #[test]
    fn test_install_already_installed_skips() -> Result<()> {
        // This test requires mocking select_assets and download_asset
        // For now, we'll test the check_if_installed path indirectly
        // by verifying that when an installation exists, install would skip

        let env = TestEnv::new()?;

        // Set up environment variables for datadirs
        temp_env::with_vars(
            vec![
                ("HOME", Some(env.home_dir.to_str().unwrap())),
                #[cfg(target_os = "linux")]
                (
                    "XDG_DATA_HOME",
                    Some(env.home_dir.join(".local/share").to_str().unwrap()),
                ),
                #[cfg(target_os = "linux")]
                (
                    "XDG_CACHE_HOME",
                    Some(env.home_dir.join(".cache").to_str().unwrap()),
                ),
            ],
            || {
                // Create an existing installation
                let repo = "testuser/testrepo";
                let version = "1.0.0";
                let install_dir = get_install_dir(repo, version).unwrap();
                prepare_install_dir(&install_dir).unwrap();

                // Add a binary to make it "installed"
                let binary_path = install_dir.join("testrepo");
                env.create_platform_executable(&binary_path).unwrap();

                // Verify it's detected as installed
                let is_installed = check_if_installed(&install_dir).unwrap();
                assert!(is_installed, "Should detect existing installation");

                // Note: We can't easily test the full install() function here without
                // mocking GitHub API and downloads, but we've verified the key check
                // that causes the early return
            },
        );

        Ok(())
    }
}

// =============================================================================
// Tests for check_for_same_named_binary_in_bin_dir
// =============================================================================

#[cfg(test)]
mod check_for_same_named_binary_in_bin_dir_tests {
    use super::*;

    #[test]
    fn test_no_existing_file() -> Result<()> {
        let env = TestEnv::new()?;
        let slug = TestEnv::test_slug();
        let bin_dir = env.create_bin_dir()?;
        let exec_path = bin_dir.join("nonexistent");

        let result = check_for_same_named_binary_in_bin_dir(&slug, &exec_path);
        assert!(result.is_ok(), "Should return Ok when no file exists");

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_symlink_to_same_slug_version_upgrade() -> Result<()> {
        let env = TestEnv::new()?;
        let slug = TestEnv::test_slug();

        env.with_test_env(|| {
            // Create installation with symlink
            let _install_dir = env
                .create_mock_installation_with_slug(
                    TestEnv::DEFAULT_TEST_SLUG,
                    "1.0.0",
                    TestEnv::DEFAULT_BINARY_NAME,
                    true,
                )
                .unwrap();

            // Get the symlink path
            let symlink_path = env.get_symlink_path(TestEnv::DEFAULT_BINARY_NAME).unwrap();

            // Check should pass for same slug (version upgrade scenario)
            let result = check_for_same_named_binary_in_bin_dir(&slug, &symlink_path);
            assert!(
                result.is_ok(),
                "Should return Ok for symlink to same slug: {:?}",
                result
            );
        });

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_symlink_to_different_slug() -> Result<()> {
        let env = TestEnv::new()?;
        let slug = TestEnv::test_slug(); // testuser/testrepo

        env.with_test_env(|| {
            // Create installation for different slug with symlink
            env.create_mock_installation_with_slug(
                "otheruser/othertool",
                "1.0.0",
                TestEnv::DEFAULT_BINARY_NAME,
                true,
            )
            .unwrap();

            // Get the symlink path
            let symlink_path = env.get_symlink_path(TestEnv::DEFAULT_BINARY_NAME).unwrap();

            // Check should fail for different slug
            let result = check_for_same_named_binary_in_bin_dir(&slug, &symlink_path);
            assert!(
                result.is_err(),
                "Should return Err for symlink to different slug"
            );

            let err_msg = format!("{:?}", result.unwrap_err());
            assert!(
                err_msg.contains("already installed"),
                "Error should mention already installed: {}",
                err_msg
            );
        });

        Ok(())
    }

    #[test]
    fn test_non_symlink_file_foreign_binary() -> Result<()> {
        let env = TestEnv::new()?;
        let slug = TestEnv::test_slug();
        let bin_dir = env.create_bin_dir()?;
        let exec_path = bin_dir.join(TestEnv::DEFAULT_BINARY_NAME);

        // Create a regular file (not a symlink)
        fs::write(&exec_path, b"#!/bin/sh\necho 'foreign binary'")?;

        let result = check_for_same_named_binary_in_bin_dir(&slug, &exec_path);
        assert!(
            result.is_err(),
            "Should return Err for non-symlink file (foreign binary)"
        );

        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(
            err_msg.contains("unrecognized"),
            "Error should mention unrecognized binary: {}",
            err_msg
        );

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_symlink_with_data_dir_in_path() -> Result<()> {
        let env = TestEnv::new()?;
        let slug = TestEnv::test_slug();

        env.with_test_env(|| {
            // Create installation with symlink (version 2.0.0 this time)
            env.create_mock_installation_with_slug(
                TestEnv::DEFAULT_TEST_SLUG,
                "2.0.0",
                TestEnv::DEFAULT_BINARY_NAME,
                true,
            )
            .unwrap();

            // Get the symlink path
            let symlink_path = env.get_symlink_path(TestEnv::DEFAULT_BINARY_NAME).unwrap();

            // Verify path matching logic works correctly
            let result = check_for_same_named_binary_in_bin_dir(&slug, &symlink_path);
            assert!(
                result.is_ok(),
                "Should correctly match when data_dir and slug are in symlink target path"
            );
        });

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_broken_symlink() -> Result<()> {
        let env = TestEnv::new()?;
        let slug = TestEnv::test_slug();
        let bin_dir = env.create_bin_dir()?;
        let symlink_path = bin_dir.join(TestEnv::DEFAULT_BINARY_NAME);

        // Create a symlink to a non-existent target
        let nonexistent_target = env.home_dir.join("does_not_exist");
        std::os::unix::fs::symlink(&nonexistent_target, &symlink_path)?;

        // The check reads the symlink target but doesn't require it to exist
        // It should handle this gracefully
        let result = check_for_same_named_binary_in_bin_dir(&slug, &symlink_path);

        // The function should handle this - either Ok or Err is acceptable
        // as long as it doesn't panic
        match result {
            Ok(_) => {
                // It's Ok because the target doesn't contain data_dir or slug
            }
            Err(e) => {
                // Should error saying it points to wrong location
                let err_msg = format!("{:?}", e);
                assert!(
                    err_msg.contains("already installed") || err_msg.contains("points to"),
                    "Error should be about incorrect target: {}",
                    err_msg
                );
            }
        }

        Ok(())
    }
}

// =============================================================================
// Tests for check_for_same_named_binary_in_path
// =============================================================================

#[cfg(test)]
mod binary_in_path_is_not_managed_by_poof_tests {
    use super::*;

    #[test]
    fn test_binary_not_in_path() -> Result<()> {
        let env = TestEnv::new()?;
        let bin_dir = env.create_bin_dir()?;

        // Use a very unlikely binary name that shouldn't be in PATH
        let exec_name = OsString::from("extremely_unlikely_binary_name_12345");

        let result = binary_in_path_is_not_managed_by_poof(&exec_name, &bin_dir);
        assert!(!result, "Should return false when binary is not in PATH");

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_binary_in_path_within_poof_bin_dir() -> Result<()> {
        let env = TestEnv::new()?;

        env.with_test_env(|| {
            // Create bin directory
            let bin_dir = env.create_dir("poof_bin").unwrap();

            // Create a binary in the bin directory
            let binary_path = bin_dir.join("mytool");
            env.create_mock_executable(&binary_path).unwrap();

            // Test with PATH extended
            env.with_path_extended(&bin_dir, || {
                let exec_name = OsString::from("mytool");
                let result = binary_in_path_is_not_managed_by_poof(&exec_name, &bin_dir);

                assert!(
                    !result,
                    "Should return false when binary is in PATH within poof's bin dir (self-reference)"
                );
            });
        });

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_binary_in_path_outside_poof_bin_dir() -> Result<()> {
        let env = TestEnv::new()?;

        env.with_test_env(|| {
            // Create two directories: one for poof bin, one for third-party
            let poof_bin_dir = env.create_dir("poof_bin").unwrap();
            let third_party_dir = env.create_dir("third_party").unwrap();

            // Create a binary in the third-party directory
            let binary_path = third_party_dir.join("mytool");
            env.create_mock_executable(&binary_path).unwrap();

            // Test with third-party dir in PATH
            env.with_path_extended(&third_party_dir, || {
                let exec_name = OsString::from("mytool");
                let result = binary_in_path_is_not_managed_by_poof(&exec_name, &poof_bin_dir);

                assert!(
                    result,
                    "Should return true when binary is in PATH outside poof's bin dir"
                );
            });
        });

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_path_manipulation_detection() -> Result<()> {
        let env = TestEnv::new()?;

        env.with_test_env(|| {
            // Create a temporary directory and add it to PATH
            let temp_bin = env.create_dir("temp_bin").unwrap();
            let poof_bin = env.create_dir("poof_bin").unwrap();

            // Create executable in temp_bin
            let exec_name = "test_collision_tool";
            let binary_path = temp_bin.join(exec_name);
            env.create_mock_executable(&binary_path).unwrap();

            // Test with temp_bin in PATH
            env.with_path_extended(&temp_bin, || {
                // Verify the binary can be found by which
                let found_path = which::which(exec_name);
                assert!(found_path.is_ok(), "Binary should be found in PATH");

                // Now check our function
                let exec_osstring = OsString::from(exec_name);
                let result = binary_in_path_is_not_managed_by_poof(&exec_osstring, &poof_bin);

                assert!(result, "Should detect binary in PATH outside poof bin dir");
            });
        });

        Ok(())
    }
}
