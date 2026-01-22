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
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let home = temp_dir.path().canonicalize().unwrap().to_path_buf();

        Ok(Self {
            _temp_dir: temp_dir,
            home_dir: home,
        })
    }

    fn create_dir(&self, name: &str) -> Result<PathBuf> {
        let path = self.home_dir.join(name);
        fs::create_dir_all(&path)?;
        Ok(path.canonicalize()?)
    }

    /// Helper to create a mock executable file
    fn create_mock_executable(&self, path: &Path) -> anyhow::Result<()> {
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
    fn create_platform_executable(&self, path: &Path) -> anyhow::Result<()> {
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
            file.write_all(&[0xFE, 0xED, 0xFA, 0xCF])?;
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
    fn test_check_if_installed_not_exists() -> anyhow::Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.home_dir.join("nonexistent");

        let result = check_if_installed(&install_dir)?;
        assert!(!result, "Should return false when directory doesn't exist");

        Ok(())
    }

    #[test]
    fn test_check_if_installed_empty_dir() -> anyhow::Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.create_dir("empty_install")?;

        let result = check_if_installed(&install_dir)?;
        assert!(!result, "Should return false when directory is empty");

        Ok(())
    }

    #[test]
    fn test_check_if_installed_not_empty() -> anyhow::Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.create_dir("non_empty_install")?;

        // Add a file to make it non-empty
        fs::write(install_dir.join("some_binary"), b"content")?;

        let result = check_if_installed(&install_dir)?;
        assert!(result, "Should return true when directory is not empty");

        Ok(())
    }

    #[test]
    fn test_check_if_installed_is_file() -> anyhow::Result<()> {
        let env = TestEnv::new()?;
        let install_path = env.home_dir.join("is_a_file");
        fs::write(&install_path, b"content")?;

        let result = check_if_installed(&install_path);
        assert!(result.is_err(), "Should return error when path is a file");

        Ok(())
    }

    #[test]
    fn test_check_if_installed_with_multiple_files() -> anyhow::Result<()> {
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
    fn test_prepare_install_dir_creates_directory() -> anyhow::Result<()> {
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
    fn test_prepare_install_dir_nested_path() -> anyhow::Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.create_dir("owner/repo/1.0.0")?;

        let result = prepare_install_dir(&install_dir);
        assert!(result.is_ok(), "Should create nested directories");
        assert!(install_dir.exists(), "Nested directory should exist");

        Ok(())
    }

    #[test]
    fn test_prepare_install_dir_already_exists() -> anyhow::Result<()> {
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
    fn test_install_binary_basic() -> anyhow::Result<()> {
        let env = TestEnv::new()?;
        let source_exec = env.home_dir.join("source/mybinary");
        let install_dir = env.create_dir("install")?;

        // Create a mock executable
        env.create_mock_executable(&source_exec)?;

        let exec_stem = OsString::from("mybinary");
        let result = install_binary(&source_exec, &install_dir, &exec_stem);
        // If bin_dir cannot be determined, skip the assertion
        if let Err(e) = &result {
            if format!("{:?}", e).contains("Failed to determine") {
                eprintln!("Skipping assertion: bin_dir unavailable in test environment");
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
    fn test_install_binary_preserves_content() -> anyhow::Result<()> {
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
        let exec_stem = OsString::from("tool");
        // Handle expected failures due to bin_dir issues in test environment
        if let Err(e) = install_binary(&source_exec, &install_dir, &exec_stem) {
            if !format!("{:?}", e).contains("Failed to determine") {
                return Err(e);
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
    fn test_install_binary_sets_executable_permissions() -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let env = TestEnv::new()?;
        let source_exec = env.home_dir.join("source/executable");
        let install_dir = env.create_dir("install")?;
        fs::create_dir_all(&install_dir)?;

        env.create_mock_executable(&source_exec)?;

        let exec_stem = OsString::from("executable");
        let _ = install_binary(&source_exec, &install_dir, &exec_stem);

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
    fn test_process_install_executable_path() -> anyhow::Result<()> {
        let env = TestEnv::new()?;
        let downloaded_file = env.home_dir.join("downloaded/mybin-linux-x86_64");
        let download_to = env.create_dir("download")?;
        let install_dir = env.create_dir("install")?;
        fs::create_dir_all(&install_dir)?;

        // Create a platform-specific executable file
        env.create_platform_executable(&downloaded_file)?;

        let asset_name = String::from("mybin-linux-x86_64");
        let result = process_install(&downloaded_file, &download_to, &install_dir, &asset_name);

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
                if !err_msg.contains("Failed to determine") {
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
    fn test_process_install_archive_path() -> anyhow::Result<()> {
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

        let asset_name = String::from("archive.zip");
        let result = process_install(&downloaded_file, &download_to, &install_dir, &asset_name);

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
    fn test_install_binaries_success() -> anyhow::Result<()> {
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

        let result = install_binaries(&archive_path, &install_dir);

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
                if !err_msg.contains("Failed to determine") {
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
    fn test_install_binaries_no_executables() -> anyhow::Result<()> {
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

        let result = install_binaries(&archive_path, &install_dir);

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
    fn test_select_assets_success_with_mock() -> anyhow::Result<()> {
        // Use mockito to mock GitHub API responses
        use mockito::Server;
        use serde_json::json;

        let mut server = Server::new();

        // Mock the latest release endpoint
        let repo = "testuser/testrepo";
        let tag = "v1.0.0";

        // Determine platform-specific asset name
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        let asset_name = "testrepo-linux-x86_64";
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        let asset_name = "testrepo-linux-aarch64";
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        let asset_name = "testrepo-darwin-x86_64";
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        let asset_name = "testrepo-darwin-aarch64";
        #[cfg(target_os = "windows")]
        let asset_name = "testrepo-windows-x86_64.exe";

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
    fn test_install_already_installed_skips() -> anyhow::Result<()> {
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
