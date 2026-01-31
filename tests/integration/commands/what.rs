//! Integration tests for the 'what' command

use assert_cmd::cargo;
use serial_test::serial;
use std::process::Command;

use crate::common::helpers::make_executable;

// Common module is included from the parent integration.rs file
use super::common::fixtures::test_env::TestFixture;
use super::common::helpers::set_test_env;

// ============================================================================
// Basic Validation Tests
// ============================================================================

#[serial]
#[test]
fn test_what_requires_repo_slug() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("what");
    set_test_env(&mut cmd, &fixture);

    let output = cmd.output()?;

    assert!(
        !output.status.success(),
        "Command should fail without repo slug"
    );

    Ok(())
}

#[serial]
#[test]
fn test_what_invalid_slug_format() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Test with invalid slug format (no slash)
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("what").arg("invalidslug");
    set_test_env(&mut cmd, &fixture);

    let output = cmd.output()?;

    assert!(
        !output.status.success(),
        "Command should fail with invalid slug format"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("USERNAME/REPO") || stderr.contains("format"),
        "Error should mention correct format: {}",
        stderr
    );

    Ok(())
}

// ============================================================================
// Non-existent Repository Tests
// ============================================================================

#[serial]
#[test]
fn test_what_nonexistent_slug() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("what").arg("nonexistent/repo");
    set_test_env(&mut cmd, &fixture);

    let output = cmd.output()?;

    assert!(
        !output.status.success(),
        "Command should fail when repository doesn't exist"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not installed"),
        "Output should indicate repository not installed: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_what_fuzzy_match_suggestion() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation for a similar repo
    fixture.create_fake_installation("testuser/testrepo", "1.0.0")?;

    // Try to query a similar but incorrect repo name
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("what").arg("testuser/testrepoo"); // Note the extra 'o'
    set_test_env(&mut cmd, &fixture);

    let output = cmd.output()?;

    assert!(
        !output.status.success(),
        "Command should fail when repository doesn't exist"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Did you mean") || stderr.contains("testuser/testrepo"),
        "Output should suggest similar repository: {}",
        stderr
    );

    Ok(())
}

// ============================================================================
// Single Version Tests
// ============================================================================

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_what_single_version_single_binary() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation with one version and one binary
    let install_dir = fixture.create_fake_installation("testuser/mytool", "1.0.0")?;

    // Verify the binary was created
    assert!(install_dir.join("mytool").exists());

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("what").arg("testuser/mytool");
    set_test_env(&mut cmd, &fixture);

    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Command should succeed: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("testuser/mytool"),
        "Output should show slug: {}",
        stdout
    );
    assert!(
        stdout.contains("1.0.0"),
        "Output should show version: {}",
        stdout
    );
    assert!(
        stdout.contains("mytool"),
        "Output should list the binary: {}",
        stdout
    );

    Ok(())
}

// ============================================================================
// Multiple Version Tests
// ============================================================================

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_what_multiple_versions_latest_selected() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create multiple versions
    fixture.create_fake_installation("testuser/multiver", "1.0.0")?;
    fixture.create_fake_installation("testuser/multiver", "2.1.0")?;
    fixture.create_fake_installation("testuser/multiver", "1.5.0")?;
    fixture.create_fake_installation("testuser/multiver", "2.0.0")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("what").arg("testuser/multiver");
    set_test_env(&mut cmd, &fixture);

    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Command should succeed: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2.1.0"),
        "Output should show latest version (2.1.0): {}",
        stdout
    );
    assert!(
        !stdout.contains("1.0.0") && !stdout.contains("1.5.0") && !stdout.contains("2.0.0"),
        "Output should not show older versions: {}",
        stdout
    );

    Ok(())
}

// ============================================================================
// Multiple Binary Tests
// ============================================================================

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_what_latest_version_multiple_binaries() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let fixture = TestFixture::new()?;

    // Create a fake installation with multiple binaries
    let install_dir = fixture.create_fake_installation("testuser/multibin", "1.0.0")?;

    // The create_fake_installation already creates one binary named "multibin"
    // Let's add more binaries
    let binary2_path = install_dir.join("tool1");
    let binary3_path = install_dir.join("tool2");

    fs::write(&binary2_path, b"#!/bin/sh\necho 'tool1'")?;
    fs::write(&binary3_path, b"#!/bin/sh\necho 'tool2'")?;

    // Make them executable
    make_executable(&binary2_path)?;
    make_executable(&binary3_path)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("what").arg("testuser/multibin");
    set_test_env(&mut cmd, &fixture);

    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Command should succeed: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("multibin"),
        "Output should list first binary: {}",
        stdout
    );
    assert!(
        stdout.contains("tool1"),
        "Output should list second binary: {}",
        stdout
    );
    assert!(
        stdout.contains("tool2"),
        "Output should list third binary: {}",
        stdout
    );

    Ok(())
}

// ============================================================================
// Semantic Versioning Tests
// ============================================================================

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_what_semantic_version_sorting() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create versions that test semantic versioning
    fixture.create_fake_installation("testuser/semver", "1.0.0")?;
    fixture.create_fake_installation("testuser/semver", "1.10.0")?;
    fixture.create_fake_installation("testuser/semver", "1.2.0")?;
    fixture.create_fake_installation("testuser/semver", "2.0.0")?;
    fixture.create_fake_installation("testuser/semver", "1.9.0")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("what").arg("testuser/semver");
    set_test_env(&mut cmd, &fixture);

    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Command should succeed: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2.0.0"),
        "Output should show latest version (2.0.0), not 1.10.0 or other versions: {}",
        stdout
    );

    Ok(())
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_what_prerelease_versions() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create versions with prerelease tags
    fixture.create_fake_installation("testuser/prerelease", "1.0.0")?;
    fixture.create_fake_installation("testuser/prerelease", "2.0.0-alpha")?;
    fixture.create_fake_installation("testuser/prerelease", "2.0.0-beta")?;
    fixture.create_fake_installation("testuser/prerelease", "2.0.0")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("what").arg("testuser/prerelease");
    set_test_env(&mut cmd, &fixture);

    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Command should succeed: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2.0.0") && !stdout.contains("alpha") && !stdout.contains("beta"),
        "Output should show stable 2.0.0, not prerelease versions: {}",
        stdout
    );

    Ok(())
}
