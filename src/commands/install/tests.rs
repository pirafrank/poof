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
            // Write a minimal but valid ELF header (20 bytes).
            // is_exec_for_current_arch checks:
            //   - ELF magic at 0x00-0x03
            //   - EI_DATA at 0x05: 1 = LE, 2 = BE (0 is invalid and causes Ok(false))
            //   - e_machine at 0x12-0x13 as u16 (byte-order from EI_DATA)
            let mut elf_stub = [0u8; 20];
            elf_stub[0..4].copy_from_slice(&magic::ELF_MAGIC);
            let (e_machine, ei_data): (u16, u8) = match std::env::consts::ARCH {
                "x86_64" => (0x003E, 1),
                "aarch64" => (0x00B7, 1),
                "x86" => (0x0003, 1),
                "arm" => (0x0028, 1),
                "riscv64" => (0x00F3, 1),
                "powerpc64" => (0x0015, 1),
                "s390x" => (0x0016, 2), // s390x is big-endian
                "loongarch64" => (0x0102, 1),
                _ => (0x0000, 1),
            };
            elf_stub[0x05] = ei_data;
            let e_machine_bytes = if ei_data == 1 {
                e_machine.to_le_bytes()
            } else {
                e_machine.to_be_bytes()
            };
            elf_stub[0x12..0x14].copy_from_slice(&e_machine_bytes);
            file.write_all(&elf_stub)?;
        }

        #[cfg(target_os = "macos")]
        {
            use std::io::Write;
            // Write a minimal valid thin 64-bit LE Mach-O stub (8 bytes).
            // is_exec_for_current_arch for the thin LE magic [0xCF,0xFA,0xED,0xFE] reads
            // the next 4 bytes as cputype (LE u32), so the correct value must follow the magic.
            // MACHO_MAGIC_NUMBERS[0] is the thin 64-bit LE magic; index 1 is the fat binary magic
            // and requires a completely different (much larger) header layout.
            file.write_all(&magic::MACHO_MAGIC_NUMBERS[0])?;
            let cputype: u32 = match std::env::consts::ARCH {
                "aarch64" => 0x0100_000C,
                "x86_64" => 0x0100_0007,
                _ => 0x0000_0000,
            };
            file.write_all(&cputype.to_le_bytes())?;
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
    fn test_install_binary_skips_shared_libraries() -> Result<()> {
        let env = TestEnv::new()?;
        let install_dir = env.create_dir("install")?;
        let slug = TestEnv::test_slug();

        for filename in ["libexample.so", "libexample.so.1", "tool.so-linux-amd64"] {
            let source = env.home_dir.join("source").join(filename);
            env.create_platform_executable(&source)?;
            assert!(is_exec_for_current_arch(&source)?);

            // The source filename must be checked even when the command name is cleaned up.
            install_binary(&slug, &source, &install_dir, &OsString::from("example"))?;
            assert_eq!(fs::read_dir(&install_dir)?.count(), 0);
        }

        Ok(())
    }

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
        if let Err(e) = &result
            && format!("{:?}", e).contains("Cannot determine")
        {
            eprintln!(
                    "Skipping test_install_binary_basic: bin_dir unavailable in CI/sandboxed environment. \
                    This is expected when HOME or XDG dirs are not properly configured."
                );
            return Ok(());
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

    #[test]
    fn duplicate_commands_preserve_existing_destinations() -> Result<()> {
        let env = TestEnv::new()?;
        env.with_test_env(|| -> Result<()> {
            let slug = TestEnv::test_slug();
            let install_dir = env.create_dir("install")?;
            let source = env.home_dir.join("source");
            env.create_platform_executable(&source)?;
            let original = install_dir.join("tool");
            fs::write(&original, b"original")?;
            install_binary(&slug, &source, &install_dir, &OsString::from("tool"))?;
            assert_eq!(fs::read(&original)?, b"original");
            fs::create_dir(install_dir.join("directory"))?;
            install_binary(&slug, &source, &install_dir, &OsString::from("directory"))?;
            assert!(install_dir.join("directory").is_dir());
            #[cfg(unix)]
            {
                let missing = env.home_dir.join("missing");
                let link = install_dir.join("dangling");
                std::os::unix::fs::symlink(&missing, &link)?;
                install_binary(&slug, &source, &install_dir, &OsString::from("dangling"))?;
                assert_eq!(fs::read_link(&link)?, missing);
                assert!(!missing.exists());
            }
            install_binary(&slug, &source, &install_dir, &OsString::new())?;
            install_binary(&slug, &source, &install_dir, &OsString::from("other"))?;
            assert_eq!(fs::read(install_dir.join("other"))?, fs::read(&source)?);
            assert!(!datadirs::get_bin_dir()
                .context("Missing test bin directory")?
                .join("tool")
                .exists());
            Ok(())
        })
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
            .expect(0)
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
            .expect(0)
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
// Tests for command name normalization
// =============================================================================

#[cfg(test)]
mod name_normalization_tests {
    use super::*;

    #[test]
    fn normalize_release_command_names() {
        for (name, expected) in [
            ("tool_v1.2.3_LINUX-x86_64", "tool"),
            ("tool-linux--_amd64__no-web", "tool-no-web"),
            ("linuxtool-arm64helper-linux-amd64", "linuxtool-arm64helper"),
            ("tool-x86-64-unknown-linux-gnu", "tool"),
            ("tool-aarch64-apple-darwin", "tool"),
            ("tool32-linux-amd64", "tool32"),
            ("tool!@#-linux-amd64", "tool!@#"),
            ("linux-amd64", "linux-amd64"),
        ] {
            assert_eq!(normalized_exec_name(name, "1.2.3"), expected, "{name}");
        }
    }

    #[test]
    fn normalize_product_prefixed_release_tags() {
        for (name, version, expected) in [
            ("bws-v2.0.0", "bws-v2.0.0", "bws"),
            ("bws-v2.0.0-linux-amd64", "bws-v2.0.0", "bws"),
            ("bws-x86_64-unknown-linux-gnu-2.0.0", "bws-v2.0.0", "bws"),
            ("bws", "bws-v2.0.0", "bws"),
            ("bws-no-web-v2.0.0-linux-amd64", "bws-v2.0.0", "bws-no-web"),
            ("bws-v2.0.0-beta.1", "bws-v2.0.0-beta.1", "bws"),
            ("bws-v2.1.0", "bws-v2.1.0", "bws"),
        ] {
            assert_eq!(normalized_exec_name(name, version), expected, "{name}");
        }
    }

    #[test]
    fn version_and_architecture_naming_regressions() {
        for (name, tag, expected) in [
            ("tool-1.2.3-linux-amd64", "tool2-v1.2.3", "tool"),
            ("tool-tool2-v1.2.3-linux-amd64", "tool2-v1.2.3", "tool"),
            ("tool-release-1.2.3", "release-1.2.3", "tool"),
            (
                "tool-v1.2.3-beta.1+build.2",
                "tool2-v1.2.3-beta.1+build.2",
                "tool",
            ),
            ("tool-1.2.30", "tool2-v1.2.3", "tool-1.2.30"),
            ("tool-nightly", "nightly", "tool"),
            ("tool2-v1.2.3-linux-amd64", "tool2-v1.2.3", "tool2"),
            ("tool-r35", "r35", "tool"),
            ("tool-v1.2-linux-amd64", "1.2", "tool"),
            ("tool-vnightly", "nightly", "tool"),
            ("gh-dash_v4.26.2_linux-arm_7", "4.26.2", "gh-dash"),
            ("python3.13", "1.2.3", "python3.13"),
            ("clang++-linux-amd64", "1.2.3", "clang++"),
            (
                "tool-unknown-feature-linux-amd64",
                "1.2.3",
                "tool-unknown-feature",
            ),
            ("apple-tool-aarch64-apple-darwin", "1.2.3", "apple-tool"),
            ("tool-gnu", "1.2.3", "tool-gnu"),
            ("tool-gnu-2.17", "1.2.3", "tool-gnu-2.17"),
            ("tool-universal-linux-amd64", "1.2.3", "tool-universal"),
            ("tool-universal-macos-1.2.3", "1.2.3", "tool"),
            (
                "tool-aarch64-unknown-linux-musl-1-2-custom",
                "1.2.3",
                "tool-custom",
            ),
            ("tool-linux-gnueabihf-feature2", "1.2.3", "tool-feature2"),
            ("tool-linux-musleabihf-feature2", "1.2.3", "tool-feature2"),
            ("tool-LINUX-X86_64-GNU-2-17-FIPS", "1.2.3", "tool-FIPS"),
        ] {
            assert_eq!(normalized_exec_name(name, tag), expected, "{name}, {tag}");
        }
    }

    #[test]
    fn asset_names_prioritize_the_first_dot_separated_component_in_assets() {
        for (name, version, expected) in [
            ("tool-linux.helper", "1.2.3", "tool"),
            ("tool_linux.helper", "1.2.3", "tool"),
            (
                "tool-custom-feature-linux.helper",
                "1.2.3",
                "tool-custom-feature",
            ),
            ("tool-1.2.3-linux-amd64", "1.2.3", "tool"),
            (
                "tool-v1.2.3-beta.1+build.2-linux-amd64",
                "tool-v1.2.3-beta.1+build.2",
                "tool",
            ),
            (
                "mc.linux-amd64.RELEASE.2025-08-13T08-35-41Z",
                "RELEASE.2025-08-13T08-35-41Z",
                "mc",
            ),
        ] {
            assert_eq!(normalized_asset_name(name, version), expected, "{name}");
        }
    }

    #[test]
    fn asset_names_first_dot_separated_component_prioritization_does_not_propagate_to_exec_names() {
        assert_eq!(normalized_exec_name("python3.13", "3.13.3"), "python3.13");
        assert_eq!(normalized_exec_name("clang++", "1.2.3"), "clang++");
        // A dot in the product prefix is not a boundary introducing the numeric release version.
        assert_eq!(
            normalized_exec_name("tool-1.2.3", "tool.v1.2.3"),
            "tool-1.2.3"
        );
        assert_eq!(
            normalized_exec_name("tool-1.2.3", "tool.1.2.3"),
            "tool-1.2.3"
        );
    }

    #[test]
    fn asset_names_test_filename_metadata_tokens() {
        assert!(!filename_metadata_tokens("tool-apple.darwin").contains(&"apple.darwin".into()));
        assert!(!filename_metadata_tokens("tool-linux.gnu-2-17").contains(&"linux.gnu-2-17".into()));
        assert!(filename_metadata_tokens("tool-linux-gnu-2.17").contains(&"linux-gnu-2.17".into()));
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
            "1.0.0",
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
            "1.0.0",
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

    /// Select assets from release fixtures, then install raw binaries and archives with distinct payloads.
    #[test]
    fn fixture_backed_release_variants_keep_distinct_commands() -> Result<()> {
        let env = TestEnv::new()?;
        env.with_test_env(|| -> Result<()> {
            for (repo, version, fixture, expected_assets) in [
                (
                    "sigstore/cosign",
                    "3.1.4",
                    include_str!("../../core/selector/tests/assets/sigstore@cosign.ron"),
                    vec![
                        ("cosign-linux-amd64", "cosign", "cosign"),
                        (
                            "cosign-linux-pivkey-pkcs11key-amd64",
                            "cosign",
                            "cosign-pivkey-pkcs11key",
                        ),
                    ],
                ),
                (
                    "zellij-org/zellij",
                    "0.45.1",
                    include_str!("../../core/selector/tests/assets/zellij-org@zellij.ron"),
                    vec![
                        (
                            "zellij-x86_64-unknown-linux-musl.tar.gz",
                            "zellij",
                            "zellij",
                        ),
                        (
                            "zellij-no-web-x86_64-unknown-linux-musl.tar.gz",
                            "zellij",
                            "zellij-no-web",
                        ),
                    ],
                ),
                (
                    "ahmetb/kubectx",
                    "0.9.5",
                    include_str!("../../core/selector/tests/assets/ahmetb@kubectx.ron"),
                    vec![
                        ("kubectx_v0.9.5_linux_x86_64.tar.gz", "kubectx", "kubectx"),
                        ("kubens_v0.9.5_linux_x86_64.tar.gz", "kubens", "kubens"),
                    ],
                ),
            ] {
                use crate::core::selector::get_triple_compatible_assets;
                use crate::models::asset_triple::AssetTriple;

                let release_assets: Vec<String> = ron::from_str(fixture)?;
                // Selection is fixed for reproducibility; executable headers match the test host.
                let triple = AssetTriple::new("linux".into(), "x86_64".into(), false);
                let assets = get_triple_compatible_assets(&release_assets, &triple, |name| name)
                    .context("No compatible fixture assets")?;
                assert_eq!(assets.len(), expected_assets.len(), "{repo}");
                let slug = Slug::new(repo)?;
                for reverse in [false, true] {
                    let install_dir = get_install_dir(repo, &format!("{version}-{reverse}"))?;
                    prepare_install_dir(&install_dir)?;
                    let mut assets = assets.clone();
                    if reverse {
                        assets.reverse();
                    }
                    let mut expected_payloads = Vec::new();
                    for asset in assets {
                        let (_, internal_name, expected) = expected_assets
                            .iter()
                            .find(|(name, _, _)| *name == asset)
                            .with_context(|| {
                                format!("Unexpected asset selected for {repo}: {asset}")
                            })?;
                        let download_dir = env.create_dir(&format!("downloads/{asset}"))?;
                        let source = download_dir.join("source");
                        env.create_platform_executable(&source)?;
                        use std::io::Write;
                        fs::OpenOptions::new()
                            .append(true)
                            .open(&source)?
                            .write_all(expected.as_bytes())?;
                        let payload = fs::read(&source)?;
                        let downloaded = download_dir.join(&asset);
                        if asset.ends_with(".tar.gz") {
                            let file = fs::File::create(&downloaded)?;
                            let gzip =
                                flate2::write::GzEncoder::new(file, flate2::Compression::default());
                            let mut archive = tar::Builder::new(gzip);
                            archive.append_path_with_name(&source, *internal_name)?;
                            archive.into_inner()?.finish()?;
                        } else {
                            fs::copy(&source, &downloaded)?;
                        }
                        process_install(
                            &slug,
                            version,
                            &downloaded,
                            &download_dir,
                            &install_dir,
                            &asset.to_string(),
                        )?;
                        expected_payloads.push((*expected, payload));
                    }
                    // Verify all contents after every asset has been processed.
                    for (expected, payload) in expected_payloads {
                        assert_eq!(fs::read(install_dir.join(expected))?, payload);
                        #[cfg(unix)]
                        assert_eq!(
                            fs::read_link(
                                datadirs::get_bin_dir()
                                    .context("Missing test bin directory")?
                                    .join(expected)
                            )?,
                            install_dir.join(expected)
                        );
                    }
                    assert_eq!(fs::read_dir(&install_dir)?.count(), 2);
                }
            }
            Ok(())
        })
    }

    #[test]
    fn product_prefixed_tag_installs_unversioned_command() -> Result<()> {
        use crate::core::selector::get_triple_compatible_assets;
        use crate::models::asset_triple::AssetTriple;

        let release_assets: Vec<String> = ron::from_str(include_str!(
            "../../core/selector/tests/assets/bitwarden@sdk-sm.ron"
        ))?;
        for (arch, musl, expected_asset) in [
            ("x86_64", false, "bws-x86_64-unknown-linux-gnu-2.0.0.zip"),
            ("x86_64", true, "bws-x86_64-unknown-linux-musl-2.0.0.zip"),
            ("aarch64", false, "bws-aarch64-unknown-linux-gnu-2.0.0.zip"),
            ("aarch64", true, "bws-aarch64-unknown-linux-musl-2.0.0.zip"),
        ] {
            let triple = AssetTriple::new("linux".into(), arch.into(), musl);
            let assets = get_triple_compatible_assets(&release_assets, &triple, |name| name)
                .context("No compatible Bitwarden release assets")?;
            assert_eq!(assets, vec![expected_asset.to_string()]);
            let asset_name = &assets[0];
            let env = TestEnv::new()?;
            env.with_test_env(|| -> Result<()> {
                let slug = Slug::new("bitwarden/sdk-sm")?;
                let version = "bws-v2.0.0";
                let download_dir = env.create_dir("downloads")?;
                let install_dir = get_install_dir("bitwarden/sdk-sm", version)?;
                prepare_install_dir(&install_dir)?;
                let source = download_dir.join("bws");
                // Selection uses a fixed target; executable bytes match the test host.
                env.create_platform_executable(&source)?;
                let downloaded = download_dir.join(asset_name);
                let file = fs::File::create(&downloaded)?;
                let mut archive = zip::ZipWriter::new(file);
                archive.start_file(
                    "bws",
                    zip::write::SimpleFileOptions::default().unix_permissions(0o755),
                )?;
                use std::io::Write;
                archive.write_all(&fs::read(&source)?)?;
                archive.finish()?;
                process_install(
                    &slug,
                    version,
                    &downloaded,
                    &download_dir,
                    &install_dir,
                    asset_name,
                )?;
                assert_eq!(fs::read(install_dir.join("bws"))?, fs::read(&source)?);
                assert_eq!(fs::read_dir(&install_dir)?.count(), 1);
                #[cfg(unix)]
                assert_eq!(
                    fs::read_link(
                        datadirs::get_bin_dir()
                            .context("Missing bin dir")?
                            .join("bws")
                    )?,
                    install_dir.join("bws")
                );
                Ok(())
            })?;
        }
        Ok(())
    }

    /// Packaging labels cannot override authoritative names inside an archive.
    #[test]
    fn archive_metadata_and_internal_names_regressions() -> Result<()> {
        let env = TestEnv::new()?;
        env.with_test_env(|| -> Result<()> {
            for (index, (asset, version, commands)) in [
                (
                    "tool-linux-x86_64-gnu-2-17.tar.gz",
                    "1.2.3",
                    vec![("tool", "tool")],
                ),
                (
                    "tool-linux-amd64-glibc-2.17.tar.gz",
                    "1.2.3",
                    vec![("tool", "tool")],
                ),
                (
                    "tool-macos-universal-1.2.3.tar.gz",
                    "1.2.3",
                    vec![("tool", "tool")],
                ),
                (
                    "tool-release-1.2.3.tar.gz",
                    "release-1.2.3",
                    vec![("tool", "tool")],
                ),
                (
                    "tool-linux-1.2.3.tar.gz",
                    "tool2-v1.2.3",
                    vec![("tool", "tool")],
                ),
                (
                    "tool-unknown-feature-linux-amd64.tar.gz",
                    "1.2.3",
                    vec![("tool", "tool-unknown-feature")],
                ),
                (
                    "tool-custom-feature-linux-amd64.tar.gz",
                    "1.2.3",
                    vec![("tool", "tool-custom-feature"), ("helper", "helper")],
                ),
                (
                    "tool-custom-feature2-linux-x86_64-gnu-2-17.tar.gz",
                    "1.2.3",
                    vec![("tool", "tool-custom-feature2")],
                ),
                (
                    "tool_no-web_linux_amd64.tar.gz",
                    "1.2.3",
                    vec![("tool", "tool_no-web")],
                ),
                (
                    "bundle.tar.gz",
                    "1.2.3",
                    vec![
                        ("clang", "clang"),
                        ("clang++", "clang++"),
                        ("python3.13", "python3.13"),
                        ("pc", "pc"),
                        ("apple", "apple"),
                        ("gnu", "gnu"),
                        ("musl", "musl"),
                        ("unknown", "unknown"),
                        ("linux", "linux"),
                        ("amd64", "amd64"),
                        ("café", "café"),
                    ],
                ),
            ]
            .into_iter()
            .enumerate()
            {
                let download = env.create_dir(&format!("regressions/{index}/download"))?;
                let install_dir =
                    get_install_dir(TestEnv::DEFAULT_TEST_SLUG, &format!("regression-{index}"))?;
                prepare_install_dir(&install_dir)?;
                let archive_path = download.join(asset);
                let gzip = flate2::write::GzEncoder::new(
                    fs::File::create(&archive_path)?,
                    flate2::Compression::default(),
                );
                let mut archive = tar::Builder::new(gzip);
                let mut expected = Vec::new();
                for (command, name) in commands {
                    let source = download.join(command);
                    env.create_platform_executable(&source)?;
                    use std::io::Write;
                    fs::OpenOptions::new()
                        .append(true)
                        .open(&source)?
                        .write_all(command.as_bytes())?;
                    archive.append_path_with_name(&source, command)?;
                    expected.push((name, fs::read(source)?));
                }
                archive.into_inner()?.finish()?;
                process_install(
                    &TestEnv::test_slug(),
                    version,
                    &archive_path,
                    &download,
                    &install_dir,
                    &asset.to_string(),
                )?;
                assert_eq!(fs::read_dir(&install_dir)?.count(), expected.len());
                for (command, payload) in expected {
                    assert_eq!(fs::read(install_dir.join(command))?, payload);
                    #[cfg(unix)]
                    assert_eq!(
                        fs::read_link(
                            datadirs::get_bin_dir()
                                .context("Missing bin dir")?
                                .join(command)
                        )?,
                        install_dir.join(command)
                    );
                }
            }
            Ok(())
        })
    }

    /// Select the real MinIO assets, then exercise the bare-executable installation path offline.
    #[test]
    fn minio_mc_fixture_installs_as_mc() -> Result<()> {
        use crate::core::selector::get_triple_compatible_assets;
        use crate::models::asset_triple::AssetTriple;

        let assets: Vec<String> = ron::from_str(include_str!(
            "../../core/selector/tests/assets/minio@mc.ron"
        ))?;
        let version = "RELEASE.2025-08-13T08-35-41Z";
        for (os, arch, expected) in [
            (
                "linux",
                "x86_64",
                "mc.linux-amd64.RELEASE.2025-08-13T08-35-41Z",
            ),
            (
                "macos",
                "aarch64",
                "mc.darwin-arm64.RELEASE.2025-08-13T08-35-41Z",
            ),
        ] {
            let triple = AssetTriple::new(os.into(), arch.into(), false);
            let selected = get_triple_compatible_assets(&assets, &triple, |asset| asset)
                .context("Missing MinIO release asset")?;
            assert_eq!(selected, vec![expected.to_string()]);
            let env = TestEnv::new()?;
            env.with_test_env(|| -> Result<()> {
                let download = env.create_dir("mc-download")?;
                let downloaded = download.join(&selected[0]);
                // Selection simulates the target above; executable bytes match the actual test host.
                env.create_platform_executable(&downloaded)?;
                let payload = fs::read(&downloaded)?;
                let install_dir = get_install_dir("minio/mc", version)?;
                prepare_install_dir(&install_dir)?;
                process_install(
                    &Slug::new("minio/mc")?,
                    version,
                    &downloaded,
                    &download,
                    &install_dir,
                    &selected[0],
                )?;
                assert_eq!(fs::read_dir(&install_dir)?.count(), 1);
                assert_eq!(fs::read(install_dir.join("mc"))?, payload);
                #[cfg(unix)]
                assert_eq!(
                    fs::read_link(
                        datadirs::get_bin_dir()
                            .context("Missing bin dir")?
                            .join("mc")
                    )?,
                    install_dir.join("mc")
                );
                Ok(())
            })?;
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
        let result = install_binaries(
            &slug,
            "1.0.0",
            temp_extract.path(),
            &install_dir,
            "archive.zip",
        );

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
        let result = install_binaries(&slug, "1.0.0", &archive_path, &install_dir, "archive.zip");

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

    #[test]
    fn archive_duplicates_are_skipped_in_sorted_order() -> Result<()> {
        let env = TestEnv::new()?;
        env.with_test_env(|| -> Result<()> {
            let extracted = env.create_dir("extracted")?;
            let install_dir = env.create_dir("install")?;
            // Create in reverse order to ensure filesystem traversal does not pick the winner.
            for (path, marker) in [
                ("b/tool-amd64", b"second"),
                ("a/tool-linux", b"first!"),
                ("other", b"other!"),
            ] {
                let path = extracted.join(path);
                env.create_platform_executable(&path)?;
                use std::io::Write;
                fs::OpenOptions::new()
                    .append(true)
                    .open(&path)?
                    .write_all(marker)?;
            }
            install_binaries(
                &TestEnv::test_slug(),
                "1.0.0",
                &extracted,
                &install_dir,
                "bundle.tar.gz",
            )?;
            assert_eq!(
                fs::read(install_dir.join("tool"))?,
                fs::read(extracted.join("a/tool-linux"))?
            );
            assert_eq!(
                fs::read(install_dir.join("other"))?,
                fs::read(extracted.join("other"))?
            );
            assert_eq!(fs::read_dir(&install_dir)?.count(), 2);
            Ok(())
        })
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
                assert!(
                    which::which("mytool").is_ok(),
                    "Binary should be found in PATH"
                );
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
}
