use super::*;
use crate::constants::{APP_NAME, DATA_SUBDIR, GITHUB_SUBDIR};
use crate::models::spell::Spell;
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
fn test_update_single_repo_on_error_with_newer_version() -> Result<()> {
    let test_env = setup_test_env()?;

    // Create fake installation with older version
    create_fake_installation(test_env.data_dir.as_path(), "testuser/testrepo", "1.0.0")?;

    let mut server = Server::new();
    // Mock GitHub API to return newer version
    let _m = mock_release_response(&mut server, "testuser/testrepo", "v2.0.0", 200);

    let server_url = server.url();
    let mut env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();
    env_vars.push(("POOF_GITHUB_API_URL", Some(server_url.as_str())));

    temp_env::with_vars(env_vars, || {
        let result = update_single_repo("testuser/testrepo");
        // install() should fail since we haven't mocked download assets
        assert!(result.is_err(), "Expected error when install() fails");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("install") || err_msg.contains("Failed"),
            "Expected error about installation, got: {}",
            err_msg
        );
    });

    Ok(())
}

#[test]
fn test_update_all_repos_with_multiple_installations() -> Result<()> {
    let test_env = setup_test_env()?;

    // Create multiple fake installations
    create_fake_installation(test_env.data_dir.as_path(), "user1/repo1", "1.0.0")?;
    create_fake_installation(test_env.data_dir.as_path(), "user2/repo2", "1.0.0")?;
    create_fake_installation(test_env.data_dir.as_path(), "user3/repo3", "1.0.0")?;

    let mut server = Server::new();
    // Mock successful responses for repo1 and repo2
    let _m1 = mock_release_response(&mut server, "user1/repo1", "v1.0.0", 200);
    let _m2 = mock_release_response(&mut server, "user2/repo2", "v1.0.0", 200);
    // Mock failure for repo3
    let _m3 = mock_release_response(&mut server, "user3/repo3", "v1.0.0", 500);

    let server_url = server.url();
    let mut env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();
    env_vars.push(("POOF_GITHUB_API_URL", Some(server_url.as_str())));

    temp_env::with_vars(env_vars, || {
        let result = update_all_repos();
        // Should fail because repo3 failed
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // Should mention the failed repo
        assert!(err_msg.contains("user3/repo3"));
        assert!(err_msg.contains("Update --all finished with errors"));
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
        assert!(err_msg.contains("Cannot parse") || err_msg.contains("semver"));
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
        assert!(err_msg.contains("Cannot parse") || err_msg.contains("semver"));
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
        assert!(err_msg.contains("Cannot get latest release") || err_msg.contains("500"));
    });

    Ok(())
}

#[test]
fn test_update_single_repo_with_spell_uses_provided_versions() -> Result<()> {
    let test_env = setup_test_env()?;

    let mut server = Server::new();
    let _m = mock_release_response(&mut server, "testuser/testrepo", "v1.0.0", 200);

    let server_url = server.url();
    let mut env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();
    env_vars.push(("POOF_GITHUB_API_URL", Some(server_url.as_str())));

    temp_env::with_vars(env_vars, || {
        // No filesystem installation is created; this only succeeds if the
        // spell-aware path uses the provided versions instead of re-reading from disk.
        let spell = Spell::new_as_string(
            "testuser/testrepo".to_string(),
            vec!["invalid-version".to_string()],
        );
        let result = update_single_repo_with_spell("testuser/testrepo", &spell);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Cannot parse") || err_msg.contains("semver"));
    });

    Ok(())
}

#[test]
fn test_process_update_with_no_arguments() -> Result<()> {
    use crate::cli::UpdateArgs;

    let args = UpdateArgs {
        repo: None,
        all: false,
    };

    let result = process_update(&args);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("No repository specified"));

    Ok(())
}

#[test]
fn test_process_update_with_all_flag() -> Result<()> {
    use crate::cli::UpdateArgs;

    let test_env = setup_test_env()?;

    let args = UpdateArgs {
        repo: None,
        all: true,
    };

    let env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();

    temp_env::with_vars(env_vars, || {
        // With empty installation, should succeed
        let result = process_update(&args);
        assert!(result.is_ok());
    });

    Ok(())
}

#[test]
fn test_process_update_with_repo_name() -> Result<()> {
    use crate::cli::UpdateArgs;

    let test_env = setup_test_env()?;

    let args = UpdateArgs {
        repo: Some("user/repo".to_string()),
        all: false,
    };

    let env_vars: Vec<(&str, Option<&str>)> = test_env
        .env_vars
        .iter()
        .map(|(k, v)| (*k, Some(v.as_str())))
        .collect();

    temp_env::with_vars(env_vars, || {
        // Repo not installed, should succeed with message
        let result = process_update(&args);
        assert!(result.is_ok());
    });

    Ok(())
}
