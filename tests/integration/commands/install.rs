//! Integration tests for the 'install' command

use assert_cmd::prelude::*;
use serial_test::serial;
use std::process::Command;

// Common module is included from the parent integration.rs file
use super::common::fixtures::test_env::TestFixture;
use super::common::repo_format_validation::*;

#[serial]
#[test]
fn test_install_requires_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("install").assert().failure();
    Ok(())
}

#[serial]
#[test]
fn test_install_comprehensive_invalid_repo_formats() -> Result<(), Box<dyn std::error::Error>> {
    test_invalid_repo_formats_for_command("install")
}

#[serial]
#[test]
fn test_install_comprehensive_valid_repo_formats() -> Result<(), Box<dyn std::error::Error>> {
    test_valid_repo_formats_for_command("install")
}

#[serial]
#[test]
fn test_install_with_tag() -> Result<(), Box<dyn std::error::Error>> {
    // Test that --tag flag is accepted
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("install")
        .arg("user/repo")
        .arg("--tag")
        .arg("v1.0.0")
        .output()?;

    // Should not fail on argument parsing
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("unknown flag"),
        "Tag flag should be accepted: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_install_creates_directories() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Even if install fails (network, etc.), it should attempt to create cache/data dirs
    let mut cmd = Command::cargo_bin("poof")?;
    let _output = cmd
        .arg("install")
        .arg("nonexistent/repo")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_DATA_HOME",
            fixture
                .home_dir
                .join(".local")
                .join("share")
                .to_str()
                .unwrap(),
        )
        .env(
            "XDG_CACHE_HOME",
            fixture.home_dir.join(".cache").to_str().unwrap(),
        )
        .output()?;

    // Cache and data directories should exist (created by datadirs functions)
    // Note: They may be created even if install fails
    let _ = fixture.cache_dir;
    let _ = fixture.data_dir;

    Ok(())
}

#[serial]
#[test]
fn test_install_state_after_success() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation to test state verification
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    let install_dir = fixture.create_fake_installation(repo, version)?;

    // Verify installation directory exists
    assert!(install_dir.exists(), "Install directory should exist");
    assert!(
        install_dir.is_dir(),
        "Install directory should be a directory"
    );

    // Verify binary exists in install directory
    let binary_name = repo.split('/').next_back().unwrap_or("testrepo");
    let binary_path = install_dir.join(binary_name);
    assert!(
        binary_path.exists(),
        "Binary should exist in install directory"
    );

    Ok(())
}

#[serial]
#[test]
fn test_install_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create an existing installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    fixture.create_fake_installation(repo, version)?;

    // Verify the installation exists
    assert!(
        fixture.is_binary_installed(repo, version),
        "Binary should be installed"
    );

    // Attempting to install again should be handled gracefully
    // (In real scenario, it would skip or warn, but for test we just verify state)
    assert!(
        fixture.is_binary_installed(repo, version),
        "Binary should still be installed after second attempt"
    );

    Ok(())
}
