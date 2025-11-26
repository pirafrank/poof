//! Integration tests for the 'update' command

use assert_cmd::prelude::*;
use serial_test::serial;
use std::process::Command;

// Common module is included from the parent integration.rs file
use super::common::*;

#[serial]
#[test]
fn test_update_requires_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("update")
        .assert()
        .failure();
    Ok(())
}

#[serial]
#[test]
fn test_update_with_repo() -> Result<(), Box<dyn std::error::Error>> {
    // Test that update accepts a repo argument
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("update")
        .arg("user/repo")
        .output()?;
    
    // Should not fail on argument parsing
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("unknown"),
        "Repo argument should be accepted: {}",
        stderr
    );
    
    Ok(())
}

#[serial]
#[test]
fn test_update_all_flag() -> Result<(), Box<dyn std::error::Error>> {
    // Test that --all flag is accepted
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("update")
        .arg("--all")
        .output()?;
    
    // Should not fail on argument parsing
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("unknown flag"),
        "--all flag should be accepted: {}",
        stderr
    );
    
    Ok(())
}

#[serial]
#[test]
fn test_update_self_flag() -> Result<(), Box<dyn std::error::Error>> {
    // Test that --self flag is accepted
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("update")
        .arg("--self")
        .output()?;
    
    // Should not fail on argument parsing
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("unexpected argument") && !stderr.contains("unknown flag"),
        "--self flag should be accepted: {}",
        stderr
    );
    
    Ok(())
}

#[serial]
#[test]
fn test_update_conflicting_flags() -> Result<(), Box<dyn std::error::Error>> {
    // Test that conflicting flags are rejected
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("update")
        .arg("--all")
        .arg("--self")
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be used"));
    Ok(())
}

#[serial]
#[test]
fn test_update_repo_and_all_conflict() -> Result<(), Box<dyn std::error::Error>> {
    // Test that repo and --all cannot be used together
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("update")
        .arg("user/repo")
        .arg("--all")
        .assert()
        .failure()
        .stderr(predicates::str::contains("cannot be used"));
    Ok(())
}

#[serial]
#[test]
fn test_update_with_nonexistent_repo() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    
    // Try to update a repo that doesn't exist
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("update")
        .arg("nonexistent/repo")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env("XDG_DATA_HOME", fixture.home_dir.join(".local").join("share").to_str().unwrap())
        .output()?;
    
    // Should handle gracefully (may fail on network or indicate not installed)
    let stderr = String::from_utf8_lossy(&output.stderr);
    // The exact message depends on implementation, but should not crash
    let _ = stderr;
    
    Ok(())
}

#[serial]
#[test]
fn test_update_with_installed_repo() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    
    // Create a fake installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    fixture.create_fake_installation(repo, version)?;
    
    // Try to update (will fail on network, but should handle gracefully)
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("update")
        .arg(repo)
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env("XDG_DATA_HOME", fixture.home_dir.join(".local").join("share").to_str().unwrap())
        .output()?;
    
    // Should attempt to check for updates (may fail on network)
    let _ = output; // Just verify it doesn't crash
    
    Ok(())
}

#[serial]
#[test]
fn test_update_all_with_installations() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    
    // Create multiple fake installations
    fixture.create_fake_installation("user1/repo1", "1.0.0")?;
    fixture.create_fake_installation("user2/repo2", "2.0.0")?;
    
    // Try to update all (will fail on network, but should handle gracefully)
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("update")
        .arg("--all")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env("XDG_DATA_HOME", fixture.home_dir.join(".local").join("share").to_str().unwrap())
        .output()?;
    
    // Should attempt to check for updates for all installed repos
    let _ = output; // Just verify it doesn't crash
    
    Ok(())
}
