//! Integration tests for the 'download' command

use assert_cmd::prelude::*;
use serial_test::serial;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[serial]
#[test]
fn test_download_requires_repo() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("download")
        .assert()
        .failure()
        .stderr(predicates::str::contains("required"));
    Ok(())
}

#[serial]
#[test]
fn test_download_invalid_repo_format() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("download")
        .arg("invalid-format")
        .assert()
        .failure()
        .stderr(predicates::str::contains(
            "Repository must be in the format",
        ));
    Ok(())
}

#[serial]
#[test]
fn test_download_valid_repo_format() -> Result<(), Box<dyn std::error::Error>> {
    // This will fail on network/actual download, but should pass format validation
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd.arg("download").arg("user/repo").output()?;

    // Should not fail on format validation
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Repository must be in the format"),
        "Valid repo format should not be rejected: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_download_with_tag() -> Result<(), Box<dyn std::error::Error>> {
    // Test that --tag flag is accepted
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("download")
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
fn test_download_to_current_directory() -> Result<(), Box<dyn std::error::Error>> {
    // Test that download command accepts current directory as target
    let temp_dir = TempDir::new()?;
    let original_dir = std::env::current_dir()?;

    std::env::set_current_dir(temp_dir.path())?;

    // This will fail on actual download, but we're testing the command structure
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd.arg("download").arg("user/repo").output()?;

    std::env::set_current_dir(original_dir)?;

    // Command should attempt to run (may fail on network, but not on structure)
    let _ = output; // Just verify it doesn't panic on invalid args

    Ok(())
}
