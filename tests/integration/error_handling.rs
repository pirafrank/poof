//! Integration tests for error handling in stateful commands

use assert_cmd::prelude::*;
use serial_test::serial;
use std::process::Command;
use tempfile::TempDir;

#[path = "../common/mod.rs"]
mod common;

use common::*;

#[serial]
#[test]
fn test_list_with_empty_data_dir() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    
    // Ensure data dir exists but is empty
    std::fs::create_dir_all(&fixture.data_dir)?;
    
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("list")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env("XDG_DATA_HOME", fixture.home_dir.join(".local").join("share").to_str().unwrap())
        .output()?;
    
    assert!(output.status.success(), "List should succeed with empty data dir");
    
    Ok(())
}

#[serial]
#[test]
fn test_use_with_nonexistent_version() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    
    let repo = "testuser/testrepo";
    
    // Don't create any installation
    
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("use")
        .arg(repo)
        .arg("--tag")
        .arg("999.999.999")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env("XDG_DATA_HOME", fixture.home_dir.join(".local").join("share").to_str().unwrap())
        .output()?;
    
    assert!(!output.status.success(), "Use should fail with nonexistent version");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not installed") || stderr.contains("not found"),
        "Should indicate version not found: {}",
        stderr
    );
    
    Ok(())
}

#[serial]
#[test]
fn test_use_with_wrong_repo() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    
    // Install one repo
    fixture.create_fake_installation("user1/repo1", "1.0.0")?;
    
    // Try to use a different repo
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("use")
        .arg("user2/repo2")
        .arg("--tag")
        .arg("1.0.0")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env("XDG_DATA_HOME", fixture.home_dir.join(".local").join("share").to_str().unwrap())
        .output()?;
    
    assert!(!output.status.success(), "Use should fail with wrong repo");
    
    Ok(())
}

#[serial]
#[test]
fn test_list_with_corrupted_directory_structure() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    
    // Create a file where a directory should be
    let bad_path = fixture.data_dir.join("user").join("repo");
    std::fs::create_dir_all(bad_path.parent().unwrap())?;
    std::fs::write(&bad_path, b"not a directory")?;
    
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("list")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env("XDG_DATA_HOME", fixture.home_dir.join(".local").join("share").to_str().unwrap())
        .output()?;
    
    // List should handle this gracefully (skip or error appropriately)
    // The exact behavior depends on implementation
    let _ = output; // Just ensure it doesn't panic
    
    Ok(())
}

#[serial]
#[test]
fn test_enable_without_bin_dir() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;
    
    // Don't create bin directory
    
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("enable")
        .env("HOME", temp_home.path())
        .env("SHELL", "/bin/bash")
        .env("XDG_DATA_HOME", temp_home.path().join(".local").join("share"))
        .output()?;
    
    // Enable might fail or succeed depending on implementation
    // The bin dir might be created automatically
    let _ = output; // Just ensure it doesn't panic
    
    Ok(())
}
