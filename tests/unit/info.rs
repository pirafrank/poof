//! Unit tests for the 'info' command

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_info_command_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("info")
        .assert()
        .success()
        .stdout(predicate::str::contains("poof"))
        .stdout(predicate::str::contains("Platform Information:"))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_info_shows_platform_information() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd.arg("info").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for required platform information fields
    assert!(
        stdout.contains("Platform Information:"),
        "Should show platform information section"
    );
    assert!(stdout.contains("OS family"), "Should show OS family");
    assert!(stdout.contains("OS type"), "Should show OS type");
    assert!(stdout.contains("OS version"), "Should show OS version");
    assert!(stdout.contains("Arch"), "Should show architecture");
    assert!(stdout.contains("Endianness"), "Should show endianness");

    Ok(())
}

#[test]
fn test_info_shows_environment_information() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd.arg("info").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for environment section
    assert!(
        stdout.contains("Environment:"),
        "Should show environment section"
    );
    assert!(stdout.contains("SHELL:"), "Should show shell information");
    assert!(stdout.contains("USER"), "Should show user information");
    assert!(stdout.contains("HOME"), "Should show home directory");
    assert!(stdout.contains("PATH"), "Should show PATH status");

    Ok(())
}

#[test]
fn test_info_shows_directory_information() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd.arg("info").output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for directories section
    assert!(
        stdout.contains("Directories:"),
        "Should show directories section"
    );
    assert!(stdout.contains("Cache dir"), "Should show cache directory");
    assert!(stdout.contains("Data dir"), "Should show data directory");
    assert!(stdout.contains("Bin dir"), "Should show bin directory");

    Ok(())
}

#[test]
fn test_info_command_no_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("info").assert().success();
    Ok(())
}

#[test]
fn test_info_command_with_extra_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    // Info command should ignore extra args or fail gracefully
    cmd.arg("info").arg("extra").assert().failure(); // clap should reject extra positional args
    Ok(())
}
