//! Integration tests for the 'list' command

use assert_cmd::prelude::*;
use serial_test::serial;
use std::process::Command;
use tempfile::TempDir;

// Common module is included from the parent integration.rs file
use super::common::*;

#[serial]
#[test]
fn test_list_with_no_installations() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("list")
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
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "List command should succeed even with no installations"
    );
    assert!(
        stdout.contains("No installed binaries found") || stdout.is_empty(),
        "Should indicate no binaries found. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_single_installation() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    fixture.create_fake_installation(repo, version)?;

    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("list")
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
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "List command should succeed");
    assert!(
        stdout.contains(repo),
        "Output should contain repository name: {}",
        stdout
    );
    assert!(
        stdout.contains(version),
        "Output should contain version: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_multiple_installations() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create multiple fake installations
    fixture.create_fake_installation("user1/repo1", "1.0.0")?;
    fixture.create_fake_installation("user1/repo1", "2.0.0")?;
    fixture.create_fake_installation("user2/repo2", "1.5.0")?;

    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("list")
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
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "List command should succeed");
    assert!(
        stdout.contains("user1/repo1"),
        "Output should contain first repo: {}",
        stdout
    );
    assert!(
        stdout.contains("user2/repo2"),
        "Output should contain second repo: {}",
        stdout
    );
    assert!(
        stdout.contains("1.0.0") && stdout.contains("2.0.0"),
        "Output should contain both versions for repo1: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_output_format() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    fixture.create_fake_installation("test/repo", "1.0.0")?;

    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("list")
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
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for table headers
    assert!(
        stdout.contains("Repository") || stdout.contains("Versions"),
        "Output should contain table headers: {}",
        stdout
    );

    Ok(())
}
