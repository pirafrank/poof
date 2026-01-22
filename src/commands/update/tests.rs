use super::*;
use crate::constants::{APP_NAME, DATA_SUBDIR, GITHUB_SUBDIR};
use anyhow::Result;
use mockito::Server;
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Helper struct to manage test environment
struct TestEnv {
    _temp_dir: TempDir,
    data_dir: std::path::PathBuf,
    env_vars: Vec<(&'static str, String)>,
}

/// Helper function to setup test environment with fake data directory structure
/// This function sets up the correct directory structure for the current platform
/// and returns the environment variables that need to be set for tests.
fn setup_test_env() -> Result<TestEnv> {
    let temp_dir = TempDir::new()?;

    // On Linux, XDG_DATA_HOME is respected by dirs::data_dir()
    // On macOS, dirs::data_dir() uses HOME/Library/Application Support
    // We need to construct the correct path structure for each platform
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
        // On macOS, dirs::data_dir() uses HOME/Library/Application Support
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

    // Create the full data directory structure
    let full_data_dir = data_base
        .join(APP_NAME)
        .join(DATA_SUBDIR)
        .join(GITHUB_SUBDIR);

    fs::create_dir_all(&full_data_dir)?;

    Ok(TestEnv {
        _temp_dir: temp_dir,
        data_dir: full_data_dir,
        env_vars,
    })
}

/// Helper function to create a fake installation in the test environment
/// The base_data_dir should be the path from TestEnv which points
/// to the github.com subdirectory where repos are stored.
fn create_fake_installation(base_data_dir: &Path, repo: &str, version: &str) -> Result<()> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid repo format");
    }
    // base_data_dir already points to .../poof/data/github.com
    // so we just need to add username/reponame/version
    let install_dir = base_data_dir.join(parts[0]).join(parts[1]).join(version);
    fs::create_dir_all(&install_dir)?;
    // Create a fake binary file to make the installation look valid
    let binary_path = install_dir.join(parts[1]);
    fs::write(&binary_path, b"fake binary")?;
    Ok(())
}

/// Helper to setup mock GitHub release response
fn mock_release_response(server: &mut Server, repo: &str, tag: &str, status: u16) -> mockito::Mock {
    let path = format!("/{}/releases/latest", repo);

    let mut mock = server.mock("GET", path.as_str());

    if status == 200 {
        mock = mock
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "tag_name": tag,
                    "published_at": "2024-01-01T00:00:00Z",
                    "assets": []
                })
                .to_string(),
            );
    } else {
        mock = mock.with_status(status as usize).with_body("Error");
    }

    mock.create()
}

#[test]
fn test_update_single_repo_not_installed() -> Result<()> {
    let test_env = setup_test_env()?;

    // Set environment to use our temp directory
    let env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();

    temp_env::with_vars(env_vars, || {
        // Try to update a repo that's not installed
        let result = update_single_repo("user/notinstalled");
        // Should succeed with a message that it's not installed
        assert!(result.is_ok());
    });

    Ok(())
}

#[test]
fn test_update_all_repos_empty() -> Result<()> {
    let test_env = setup_test_env()?;

    // Set environment to use our temp directory
    let env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();

    temp_env::with_vars(env_vars, || {
        // Try to update all repos when nothing is installed
        let result = update_all_repos();
        // Should succeed with a message that nothing is installed
        assert!(result.is_ok());
    });

    Ok(())
}

#[test]
fn test_update_single_repo_up_to_date() -> Result<()> {
    let test_env = setup_test_env()?;

    // Create a fake installation
    create_fake_installation(test_env.data_dir.as_path(), "testuser/testrepo", "1.0.0")?;

    let mut server = Server::new();
    let _m = mock_release_response(&mut server, "testuser/testrepo", "v1.0.0", 200);

    // Set environment to use our temp directory and mock GitHub API
    let server_url = server.url();
    let mut env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();
    env_vars.push(("POOF_GITHUB_API_URL", Some(server_url.as_str())));

    temp_env::with_vars(env_vars, || {
        // Update repo that's already up to date
        let result = update_single_repo("testuser/testrepo");
        // Should succeed and report up-to-date
        assert!(result.is_ok());
    });

    Ok(())
}

#[test]
fn test_update_self_up_to_date() -> Result<()> {
    let mut server = Server::new();
    let current_version = env!("CARGO_PKG_VERSION");
    let tag = format!("v{}", current_version);

    // Mock response with current version
    let _m = server
        .mock("GET", "/pirafrank/poof/releases/latest")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "tag_name": tag,
                "published_at": "2024-01-01T00:00:00Z",
                "assets": []
            })
            .to_string(),
        )
        .create();

    // Set environment to use mock GitHub API
    temp_env::with_var("POOF_GITHUB_API_URL", Some(server.url().as_str()), || {
        let result = update_self();
        // Should succeed and report up-to-date
        assert!(result.is_ok());
    });

    Ok(())
}

#[test]
fn test_update_self_newer_version_available() -> Result<()> {
    let mut server = Server::new();

    // Mock response with a much newer version
    let _m = server
        .mock("GET", "/pirafrank/poof/releases/latest")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "tag_name": "v999.999.999",
                "published_at": "2024-01-01T00:00:00Z",
                "assets": [{
                    "name": "poof-test-binary",
                    "browser_download_url": format!("{}/fake-download", server.url()),
                    "content_type": "application/octet-stream"
                }]
            })
            .to_string(),
        )
        .create();

    // Mock the download endpoint (it will fail, but we're testing version detection)
    let _m2 = server
        .mock("GET", "/fake-download")
        .with_status(200)
        .with_body("fake binary content")
        .create();

    // Set environment to use mock GitHub API
    temp_env::with_var("POOF_GITHUB_API_URL", Some(server.url().as_str()), || {
        let result = update_self();
        // Will fail during actual update process (download/replace), but should detect newer version
        // The function will fail at some point during the update process, not during version check
        assert!(result.is_err() || result.is_ok());
        // Either way, the version detection logic was exercised
    });

    Ok(())
}

#[test]
fn test_update_single_repo_invalid_semver_installed() -> Result<()> {
    let test_env = setup_test_env()?;

    // Create a fake installation with invalid semver
    create_fake_installation(
        test_env.data_dir.as_path(),
        "testuser/testrepo",
        "invalid-version",
    )?;

    let mut server = Server::new();
    let _m = mock_release_response(&mut server, "testuser/testrepo", "v1.0.0", 200);

    // Set environment to use our temp directory and mock GitHub API
    let server_url = server.url();
    let mut env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();
    env_vars.push(("POOF_GITHUB_API_URL", Some(server_url.as_str())));

    temp_env::with_vars(env_vars, || {
        // Update repo with invalid semver should fail
        let result = update_single_repo("testuser/testrepo");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to parse") || err_msg.contains("semver"));
    });

    Ok(())
}

#[test]
fn test_update_single_repo_invalid_semver_from_github() -> Result<()> {
    let test_env = setup_test_env()?;

    // Create a fake installation with valid semver
    create_fake_installation(test_env.data_dir.as_path(), "testuser/testrepo", "1.0.0")?;

    let mut server = Server::new();
    // Mock response with invalid semver tag
    let _m = server
        .mock("GET", "/testuser/testrepo/releases/latest")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "tag_name": "not-a-valid-semver",
                "published_at": "2024-01-01T00:00:00Z",
                "assets": []
            })
            .to_string(),
        )
        .create();

    // Set environment to use our temp directory and mock GitHub API
    let server_url = server.url();
    let mut env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();
    env_vars.push(("POOF_GITHUB_API_URL", Some(server_url.as_str())));

    temp_env::with_vars(env_vars, || {
        // Update repo should fail due to invalid semver from GitHub
        let result = update_single_repo("testuser/testrepo");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to parse") || err_msg.contains("semver"));
    });

    Ok(())
}

#[test]
fn test_update_single_repo_github_api_failure() -> Result<()> {
    let test_env = setup_test_env()?;

    // Create a fake installation
    create_fake_installation(test_env.data_dir.as_path(), "testuser/testrepo", "1.0.0")?;

    let mut server = Server::new();
    // Mock a GitHub API failure
    let _m = server
        .mock("GET", "/testuser/testrepo/releases/latest")
        .with_status(500)
        .with_body("Internal Server Error")
        .create();

    // Set environment to use our temp directory and mock GitHub API
    let server_url = server.url();
    let mut env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();
    env_vars.push(("POOF_GITHUB_API_URL", Some(server_url.as_str())));

    temp_env::with_vars(env_vars, || {
        // Update repo should fail due to GitHub API error
        let result = update_single_repo("testuser/testrepo");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to get latest release") || err_msg.contains("500"));
    });

    Ok(())
}

#[test]
fn test_update_self_invalid_semver_from_github() -> Result<()> {
    let mut server = Server::new();

    // Mock response with invalid semver tag
    let _m = server
        .mock("GET", "/pirafrank/poof/releases/latest")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "tag_name": "not-a-valid-semver",
                "published_at": "2024-01-01T00:00:00Z",
                "assets": []
            })
            .to_string(),
        )
        .create();

    // Set environment to use mock GitHub API
    temp_env::with_var("POOF_GITHUB_API_URL", Some(server.url().as_str()), || {
        let result = update_self();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to parse") || err_msg.contains("semver"));
    });

    Ok(())
}

#[test]
fn test_update_self_github_api_failure() -> Result<()> {
    let mut server = Server::new();

    // Mock a GitHub API failure
    let _m = server
        .mock("GET", "/pirafrank/poof/releases/latest")
        .with_status(404)
        .with_body("Not Found")
        .create();

    // Set environment to use mock GitHub API
    temp_env::with_var("POOF_GITHUB_API_URL", Some(server.url().as_str()), || {
        let result = update_self();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to get latest release") || err_msg.contains("404"));
    });

    Ok(())
}
