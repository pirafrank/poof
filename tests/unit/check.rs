//! Unit tests for the 'check' command

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_check_command_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("check")
        .assert()
        .success();
    Ok(())
}

#[test]
fn test_check_warns_when_not_in_path() -> Result<(), Box<dyn std::error::Error>> {
    // Set up a temporary environment where bin dir is not in PATH
    let temp_dir = TempDir::new()?;
    let original_home = std::env::var("HOME").ok();
    let original_path = std::env::var("PATH").ok();
    
    std::env::set_var("HOME", temp_dir.path());
    std::env::set_var("XDG_DATA_HOME", temp_dir.path().join(".local").join("share"));
    std::env::set_var("PATH", "/usr/bin:/bin"); // PATH without poof's bin dir
    
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd.arg("check").output()?;
    
    // Restore environment
    if let Some(home) = original_home {
        std::env::set_var("HOME", home);
    }
    if let Some(path) = original_path {
        std::env::set_var("PATH", path);
    }
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Check should warn when not in PATH
    assert!(
        stderr.contains("not found in PATH") || stderr.contains("not the first in PATH"),
        "Should warn about PATH when bin directory is not in PATH"
    );
    
    Ok(())
}

#[test]
fn test_check_command_no_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("check")
        .assert()
        .success();
    Ok(())
}

#[test]
fn test_check_command_with_extra_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    // Check command should ignore extra args or fail gracefully
    cmd.arg("check")
        .arg("extra")
        .assert()
        .failure(); // clap should reject extra positional args
    Ok(())
}
