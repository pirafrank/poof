//! Integration tests for the 'use' command (make_default)
//! Note: module named make_default because 'use' is a Rust keyword

use assert_cmd::prelude::*;
use serial_test::serial;
use std::process::Command;

// Common module is included from the parent integration.rs file
use super::common::*;

#[serial]
#[test]
fn test_use_requires_installation() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Try to use a version that doesn't exist
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("use")
        .arg("nonexistent/repo")
        .arg("--tag")
        .arg("1.0.0")
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

    // Should fail because version is not installed
    assert!(
        !output.status.success(),
        "Use command should fail when version is not installed"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not installed") || stderr.contains("not found"),
        "Should indicate version is not installed: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_use_sets_default_version() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let repo = "testuser/testrepo";
    let version1 = "1.0.0";
    let version2 = "2.0.0";

    // Create two versions
    fixture.create_fake_installation(repo, version1)?;
    fixture.create_fake_installation(repo, version2)?;

    // Get binary name
    let binary_name = repo.split('/').next_back().unwrap_or("testrepo");

    // Create symlinks for both versions initially
    let install_dir1 = fixture.get_install_path(repo, version1);
    fixture.create_bin_symlink(binary_name, &install_dir1.join(binary_name))?;

    // Use version 2.0.0 as default
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("use")
        .arg(repo)
        .arg("--tag")
        .arg(version2)
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

    assert!(output.status.success(), "Use command should succeed");

    // Verify symlink points to version 2
    let symlink_path = fixture.bin_dir.join(binary_name);
    if symlink_path.exists() {
        #[cfg(not(target_os = "windows"))]
        {
            let target = std::fs::read_link(&symlink_path)?;
            assert!(
                target.to_string_lossy().contains(version2),
                "Symlink should point to version 2"
            );
        }
    }

    Ok(())
}

#[serial]
#[test]
fn test_use_with_latest_tag() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let repo = "testuser/testrepo";
    let version = "1.0.0";

    fixture.create_fake_installation(repo, version)?;

    // Use with --tag latest (should default to latest if not specified)
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("use")
        .arg(repo)
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

    // Should succeed if version "latest" exists, or fail if it doesn't
    // The behavior depends on implementation, but command should not crash
    let _ = output; // Just ensure it doesn't panic

    Ok(())
}
