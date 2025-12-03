//! Integration tests for the 'download' command

use assert_cmd::{assert::OutputAssertExt, cargo};
use serial_test::serial;
use std::process::Command;

// Common module is included from the parent integration.rs file
use super::common::repo_format_validation::*;

#[serial]
#[test]
fn test_download_requires_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("download").assert().failure();
    Ok(())
}

#[serial]
#[test]
fn test_download_comprehensive_invalid_repo_formats() -> Result<(), Box<dyn std::error::Error>> {
    test_invalid_repo_formats_for_command("download")
}

#[serial]
#[test]
fn test_download_comprehensive_valid_repo_formats() -> Result<(), Box<dyn std::error::Error>> {
    test_valid_repo_formats_for_command("download")
}

#[serial]
#[test]
fn test_download_with_tag() -> Result<(), Box<dyn std::error::Error>> {
    // Test that --tag flag is accepted
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
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
