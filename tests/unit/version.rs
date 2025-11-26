//! Unit tests for the 'version' command

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_version_command_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")))
        .stdout(predicate::str::contains("Commit"))
        .stdout(predicate::str::contains("Build Date"))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_version_output_format() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd.arg("version").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check that output contains expected fields
    assert!(stdout.contains("Version"), "Output should contain 'Version'");
    assert!(stdout.contains("Commit"), "Output should contain 'Commit'");
    assert!(stdout.contains("Build Date"), "Output should contain 'Build Date'");
    
    // Check that version number is present
    assert!(
        stdout.contains(env!("CARGO_PKG_VERSION")),
        "Output should contain package version"
    );
    
    Ok(())
}

#[test]
fn test_version_command_no_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("version")
        .assert()
        .success();
    Ok(())
}

#[test]
fn test_version_command_with_extra_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    // Version command should ignore extra args or fail gracefully
    cmd.arg("version")
        .arg("extra")
        .assert()
        .failure(); // clap should reject extra positional args
    Ok(())
}
