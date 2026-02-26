use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use log::debug;
use log::{error, info};

use crate::files::datadirs;
use crate::files::filesys;
use crate::files::utils::find_similar_repo;
use crate::utils::semver::SemverSort;

/// Check if a repository is installed, providing helpful error messages if not.
/// Returns the path to the repository's versions directory.
fn check_repo_installed(repo: &str) -> Result<PathBuf> {
    let data_dir = datadirs::get_data_dir().context("Cannot get data directory")?;
    let versions_dir = datadirs::get_versions_nest(&data_dir, repo);

    if !versions_dir.exists() {
        // Try fuzzy finding a similar named installed repository
        if let Some(similar_repo) = find_similar_repo(&data_dir, repo) {
            error!(
                "It looks like '{}' is not installed. Did you mean: {}",
                repo, similar_repo
            );
        } else {
            error!("It looks like '{}' is not installed. Typo?", repo);
        }
        bail!("Repository '{}' not found", repo);
    }

    Ok(versions_dir)
}

/// Get the latest installed version for a repository.
/// Returns the version string of the latest version based on semver sorting.
pub(crate) fn get_latest_version(repo: &str) -> Result<String> {
    let versions_dir = check_repo_installed(repo).with_context(|| {
        error!("Install it using 'poof install {}'", repo);
        format!("Failed to find repository '{}'", repo)
    })?;

    // Read all version subdirectories
    let entries = std::fs::read_dir(&versions_dir)
        .with_context(|| format!("Cannot read versions directory for '{}'", repo))?;

    let mut versions: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                if let Some(version_name) = entry.file_name().to_str() {
                    versions.push(version_name.to_string());
                }
            }
        }
    }

    // Check if any versions were found
    if versions.is_empty() {
        error!(
            "No versions found for '{}'. Repository may be corrupted or not installed.",
            repo
        );
        error!("Install it using 'poof install {}'", repo);
        bail!("No versions found for '{}'", repo);
    }

    // Sort versions using semantic versioning
    versions.sort_semver();

    // Get the latest version (last element after sorting)
    let latest_version = versions
        .last()
        .expect("versions is non-empty after check")
        .clone();

    Ok(latest_version)
}

/// Returns the data-directory path for the given repo/version, checking that it exists.
fn get_installed_dir(repo: &str, version: &str) -> Result<PathBuf> {
    // Check repository exists
    check_repo_installed(repo).with_context(|| {
        error!("Check installed binaries using 'list' command.");
        format!("Failed to find repository '{}'", repo)
    })?;

    let data_dir = datadirs::get_data_dir().context("Cannot get data directory")?;
    let installed_version_dir = datadirs::get_binary_nest(&data_dir, repo, version);
    if !installed_version_dir.exists() {
        error!(
            "Version {} of repository '{}' is not installed. Typo?",
            version, repo
        );
        error!("Check installed versions using 'list' command.");
        bail!("Version {} of repository '{}' not found", version, repo);
    }

    Ok(installed_version_dir)
}

/// Set a specific (or the latest) installed version of `repo` as the default.
///
/// Updates the symlinks in the bin directory to point to the requested version.
/// When `version` is `None`, the highest semantically-versioned installed release
/// is selected automatically via [`get_latest_version`].
pub fn set_default(repo: &str, version: Option<&str>) -> Result<()> {
    // Resolve version: use provided version or get latest
    let resolved_version = match version {
        Some(v) => v.to_string(),
        None => {
            let latest = get_latest_version(repo).with_context(|| {
                format!("Failed to find the newest installed version for '{}'", repo)
            })?;
            debug!("Found {} to be the newest installed version", latest);
            latest
        }
    };

    // Get the installed directory for the specified repo and version
    let install_dir = get_installed_dir(repo, &resolved_version)?;
    // Get the bin directory
    let bin_dir = datadirs::get_bin_dir().context("Cannot get bin directory")?;

    // List of binaries to set as default
    let mut binaries: Vec<String> = Vec::new();
    // Process each binary in wanted_dir
    for path in filesys::find_exec_files_in_dir(&install_dir) {
        // Skip non-executable files (they all should be since they have
        // been installed, but just in case).
        // Note: Windows does not require setting executable permissions
        #[cfg(not(target_os = "windows"))]
        {
            if !filesys::is_executable(&path) {
                continue;
            }
            // Get exec filename
            let Some(file_name) = path.file_name() else {
                continue;
            };
            binaries.push(file_name.to_string_lossy().to_string());
            // make exec available in PATH, overwriting any existing symlink
            let symlink_path = bin_dir.join(file_name);
            filesys::create_symlink(&path, &symlink_path, true)
                .map_err(anyhow::Error::msg)
                .with_context(|| {
                    format!(
                        "Cannot create symlink from {} to {}",
                        path.display(),
                        symlink_path.display()
                    )
                })?;
        }
    }
    info!("Version {} set as default for:", resolved_version);
    for binary in binaries {
        info!("✓ {}", binary);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{APP_NAME, DATA_SUBDIR, GITHUB_SUBDIR};
    use std::path::Path;
    use tempfile::TempDir;

    /// Holds a live temp directory and the env vars required to point
    /// `dirs::data_dir()` at it for the duration of each test.
    struct TestEnv {
        _temp_dir: TempDir,
        data_dir: std::path::PathBuf,
        env_vars: Vec<(&'static str, String)>,
    }

    /// Create a temporary directory tree that mirrors the poof data layout and
    /// return the env vars needed to make `dirs::data_dir()` resolve into it.
    fn setup_test_env() -> TestEnv {
        let temp_dir = TempDir::new().unwrap();

        #[cfg(target_os = "linux")]
        let (data_base, env_vars) = {
            let data_base = temp_dir.path().join("data");
            let vars = vec![
                ("HOME", temp_dir.path().to_str().unwrap().to_string()),
                ("XDG_DATA_HOME", data_base.to_str().unwrap().to_string()),
            ];
            (data_base, vars)
        };

        #[cfg(target_os = "macos")]
        let (data_base, env_vars) = {
            let data_base = temp_dir.path().join("Library").join("Application Support");
            let vars = vec![("HOME", temp_dir.path().to_str().unwrap().to_string())];
            (data_base, vars)
        };

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        let (data_base, env_vars) = {
            let data_base = temp_dir.path().join("data");
            let vars = vec![("HOME", temp_dir.path().to_str().unwrap().to_string())];
            (data_base, vars)
        };

        let data_dir = data_base
            .join(APP_NAME)
            .join(DATA_SUBDIR)
            .join(GITHUB_SUBDIR);
        std::fs::create_dir_all(&data_dir).unwrap();

        TestEnv {
            _temp_dir: temp_dir,
            data_dir,
            env_vars,
        }
    }

    /// Helper to create a version directory under the test data tree.
    fn create_version_dir(data_dir: &Path, repo: &str, version: &str) {
        let separator = std::path::MAIN_SEPARATOR.to_string();
        let version_dir = data_dir.join(repo.replace('/', &separator)).join(version);
        std::fs::create_dir_all(&version_dir).unwrap();
    }

    #[test]
    fn test_get_latest_version_with_semver_sorting() {
        let test_env = setup_test_env();
        let env_vars: Vec<(&str, Option<&str>)> = test_env
            .env_vars
            .iter()
            .map(|(k, v)| (*k, Some(v.as_str())))
            .collect();

        let repo = "testuser/testrepo";
        create_version_dir(&test_env.data_dir, repo, "1.2.0");
        create_version_dir(&test_env.data_dir, repo, "2.0.0");
        create_version_dir(&test_env.data_dir, repo, "1.10.0");

        temp_env::with_vars(env_vars, || {
            let result = get_latest_version(repo);
            assert!(result.is_ok(), "Should successfully get latest version");
            assert_eq!(
                result.unwrap(),
                "2.0.0",
                "Should return 2.0.0 as the latest version based on semver sorting"
            );
        });
    }

    #[test]
    fn test_get_latest_version_repo_not_installed() {
        let test_env = setup_test_env();
        let env_vars: Vec<(&str, Option<&str>)> = test_env
            .env_vars
            .iter()
            .map(|(k, v)| (*k, Some(v.as_str())))
            .collect();

        temp_env::with_vars(env_vars, || {
            let result = get_latest_version("nonexistent/repo");
            assert!(result.is_err(), "Should fail for non-existent repository");
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("not found") || error_msg.contains("nonexistent/repo"),
                "Error message should indicate repository not found: {}",
                error_msg
            );
        });
    }

    #[test]
    fn test_get_latest_version_no_versions() {
        let test_env = setup_test_env();
        let env_vars: Vec<(&str, Option<&str>)> = test_env
            .env_vars
            .iter()
            .map(|(k, v)| (*k, Some(v.as_str())))
            .collect();

        let repo = "testuser/testrepo";
        let separator = std::path::MAIN_SEPARATOR.to_string();
        let repo_dir = test_env.data_dir.join(repo.replace('/', &separator));
        std::fs::create_dir_all(&repo_dir).unwrap();

        temp_env::with_vars(env_vars, || {
            let result = get_latest_version(repo);
            assert!(
                result.is_err(),
                "Should fail when repository has no versions"
            );
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("No versions found") || error_msg.contains("no versions"),
                "Error message should indicate no versions found: {}",
                error_msg
            );
        });
    }

    #[test]
    fn test_get_latest_version_single_version() {
        let test_env = setup_test_env();
        let env_vars: Vec<(&str, Option<&str>)> = test_env
            .env_vars
            .iter()
            .map(|(k, v)| (*k, Some(v.as_str())))
            .collect();

        let repo = "testuser/testrepo";
        create_version_dir(&test_env.data_dir, repo, "1.0.0");

        temp_env::with_vars(env_vars, || {
            let result = get_latest_version(repo);
            assert!(result.is_ok(), "Should successfully get latest version");
            assert_eq!(
                result.unwrap(),
                "1.0.0",
                "Should return the only version available"
            );
        });
    }

    #[test]
    fn test_get_latest_version_with_prerelease() {
        let test_env = setup_test_env();
        let env_vars: Vec<(&str, Option<&str>)> = test_env
            .env_vars
            .iter()
            .map(|(k, v)| (*k, Some(v.as_str())))
            .collect();

        let repo = "testuser/testrepo";
        create_version_dir(&test_env.data_dir, repo, "1.0.0");
        create_version_dir(&test_env.data_dir, repo, "2.0.0-beta.1");
        create_version_dir(&test_env.data_dir, repo, "1.5.0");

        temp_env::with_vars(env_vars, || {
            let result = get_latest_version(repo);
            assert!(result.is_ok(), "Should successfully get latest version");
            assert_eq!(
                result.unwrap(),
                "2.0.0-beta.1",
                "Should correctly handle pre-release versions in semver sorting"
            );
        });
    }
}
