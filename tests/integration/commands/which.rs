//! Integration tests for the 'which' command

use assert_cmd::cargo;
use serial_test::serial;
use std::process::Command;

// Common module is included from the parent integration.rs file
use super::common::fixtures::test_env::TestFixture;
use super::common::helpers::set_test_env;

/// Helper function to create a managed symlink for testing
/// Creates a fake installation and symlinks a binary from it to bin_dir
#[cfg(not(target_os = "windows"))]
fn create_managed_symlink(
    fixture: &TestFixture,
    binary_name: &str,
    repo: &str,
    version: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    // Create fake installation
    let install_dir = fixture.create_fake_installation(repo, version)?;

    // The installation creates a binary with the repo name (last part)
    // We need to find what binary was created or use the provided name
    let repo_binary_name = repo.split('/').next_back().unwrap_or(binary_name);
    let binary_in_install = install_dir.join(repo_binary_name);

    // If the binary doesn't exist, create it with the provided name
    if !binary_in_install.exists() {
        // Create a new binary with the desired name
        fs::write(
            install_dir.join(binary_name),
            b"#!/bin/sh\necho 'test binary'",
        )?;
        let mut perms = fs::metadata(install_dir.join(binary_name))?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(install_dir.join(binary_name), perms)?;
    }

    // Create symlink in bin_dir
    let source = if binary_in_install.exists() {
        binary_in_install
    } else {
        install_dir.join(binary_name)
    };
    let symlink_path = fixture.bin_dir.join(binary_name);
    std::os::unix::fs::symlink(&source, &symlink_path)?;

    Ok(symlink_path)
}

// ============================================================================
// Basic Validation Tests
// ============================================================================

#[serial]
#[test]
fn test_which_requires_binary_name() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which");

    let output = cmd.output()?;

    assert!(
        !output.status.success(),
        "Command should fail without binary name"
    );

    Ok(())
}

// ============================================================================
// Non-existent Binary Tests
// ============================================================================

#[serial]
#[test]
fn test_which_nonexistent_binary() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which").arg("nonexistent_binary");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Command should succeed even when binary doesn't exist"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "Output should indicate binary not found: {}",
        stderr
    );

    Ok(())
}

// ============================================================================
// Successful Lookup Tests
// ============================================================================

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_which_with_valid_symlink() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let binary_name = "mybin";
    let repo = "someuser/somerepo";
    let version = "2.5.3";

    let symlink_path = create_managed_symlink(&fixture, binary_name, repo, version)?;

    assert!(symlink_path.exists(), "Symlink should exist");

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which").arg(binary_name);
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(output.status.success(), "Command should succeed");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should show the slug (username/reponame)
    assert!(
        stderr.contains("someuser/somerepo"),
        "Output should contain the repository slug: {}",
        stderr
    );

    // Should also show version information
    assert!(
        stderr.contains("2.5.3") || stderr.contains("version"),
        "Output should contain version information: {}",
        stderr
    );

    Ok(())
}

// ============================================================================
// Non-Symlink Binary Tests
// ============================================================================

#[serial]
#[test]
fn test_which_regular_file_not_symlink() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let fixture = TestFixture::new()?;
    let binary_name = "regular_file";
    let file_path = fixture.bin_dir.join(binary_name);

    // Create a regular file (not a symlink)
    fs::write(&file_path, b"#!/bin/sh\necho 'regular file'")?;

    assert!(file_path.exists(), "File should exist");

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which").arg(binary_name);
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Command should succeed but indicate it's not managed"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not a symlink") || stderr.contains("foreign binary"),
        "Output should indicate binary is not a symlink or is foreign: {}",
        stderr
    );

    Ok(())
}

// ============================================================================
// Edge Cases
// ============================================================================

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_which_broken_symlink() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let binary_name = "broken_link";
    let symlink_path = fixture.bin_dir.join(binary_name);

    // Create a symlink pointing to a non-existent target within the data directory
    // This simulates a broken poof-managed binary
    let nonexistent_target = fixture
        .data_dir
        .join("github.com")
        .join("user")
        .join("repo")
        .join("1.0.0")
        .join("binary");
    std::os::unix::fs::symlink(&nonexistent_target, &symlink_path)?;

    // Verify the symlink metadata exists (even though target doesn't)
    assert!(
        symlink_path.symlink_metadata().is_ok(),
        "Symlink should exist (even if broken)"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which").arg(binary_name);
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // The command will report "not found" because .exists() returns false for broken symlinks
    // This is actually correct behavior - a broken symlink is effectively not usable
    assert!(
        output.status.success(),
        "Command should succeed even with broken symlink"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "Should indicate binary not found for broken symlink: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_which_symlink_outside_data_dir() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let fixture = TestFixture::new()?;
    let binary_name = "foreign_binary";
    let symlink_path = fixture.bin_dir.join(binary_name);

    // Create a target outside the data directory
    let external_target = fixture.home_dir.join("external_bin");
    fs::write(&external_target, b"#!/bin/sh\necho 'external'")?;

    // Create symlink pointing to external target
    std::os::unix::fs::symlink(&external_target, &symlink_path)?;

    assert!(symlink_path.exists(), "Symlink should exist");

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which").arg(binary_name);
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // The command should fail or return an error for external binaries
    // since the implementation bails when it can't extract the slug
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either the command fails or shows an error message
    assert!(
        !output.status.success()
            || stderr.contains("Cannot determine")
            || stderr.contains("Internal error"),
        "Should indicate error for external binary. stderr: {}, exit: {}",
        stderr,
        output.status
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_which_output_format() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let binary_name = "testbin";
    let repo = "user/repo";
    let version = "1.0.1";

    create_managed_symlink(&fixture, binary_name, repo, version)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which").arg(binary_name);
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(output.status.success(), "Command should succeed");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check output format matches expected pattern
    assert!(
        stderr.contains("is provided by"),
        "Output should contain 'is provided by': {}",
        stderr
    );

    assert!(
        stderr.contains(binary_name),
        "Output should contain binary name: {}",
        stderr
    );

    assert!(
        stderr.contains("user/repo"),
        "Output should contain the repository slug: {}",
        stderr
    );

    assert!(
        stderr.contains("version") || stderr.contains("1.0.1"),
        "Output should contain version information: {}",
        stderr
    );

    Ok(())
}
