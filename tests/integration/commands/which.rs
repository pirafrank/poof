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

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show the slug (username/reponame) in format "binary: repo version"
    assert!(
        stdout.contains("someuser/somerepo"),
        "Output should contain the repository slug: {}",
        stdout
    );

    // Should also show version information
    assert!(
        stdout.contains("2.5.3"),
        "Output should contain version information: {}",
        stdout
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
        "Command should succeed but indicate binary not found"
    );

    // New implementation: the which command now searches the data directory,
    // not the bin directory. A file in the bin directory that's not managed
    // by poof (not in data dir) will be reported as "not found".
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "Output should indicate binary is not found in installed repositories: {}",
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

    // New implementation: the which command now searches the data directory,
    // not the bin directory. A broken symlink will be correctly identified as not found
    // because the binary doesn't exist in the data directory.
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

    // New implementation: the which command now searches the data directory,
    // not the bin directory. A symlink pointing outside the data directory
    // won't be found since the binary doesn't exist in the data directory.
    assert!(
        output.status.success(),
        "Command should succeed but indicate binary not found"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found"),
        "Should indicate not found for external binary. stderr: {}, exit: {}",
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

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains(binary_name),
        "Output should contain binary name: {}",
        stdout
    );

    assert!(
        stdout.contains("user/repo"),
        "Output should contain the repository slug: {}",
        stdout
    );

    assert!(
        stdout.contains("1.0.1"),
        "Output should contain version information: {}",
        stdout
    );

    Ok(())
}

// ============================================================================
// Unlink Scenario Tests
// ============================================================================

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_which_after_unlink() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let fixture = TestFixture::new()?;
    let binary_name = "unlinktest";
    let repo = "testuser/testbin";
    let version = "2.0.0";

    // Create a fake installation (binary in data directory)
    let install_dir = fixture.create_fake_installation(repo, version)?;

    // Create the binary with the correct name
    let binary_path = install_dir.join(binary_name);
    fixture.create_executable_with_perms(&binary_path, b"#!/bin/sh\necho 'test binary'")?;

    // Create symlink in bin_dir (simulating what the install command does)
    let symlink_path = fixture.bin_dir.join(binary_name);
    std::os::unix::fs::symlink(&binary_path, &symlink_path)?;

    // Verify which works with the symlink present
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which").arg(binary_name);
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(output.status.success(), "Which should work with symlink");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("testuser/testbin"),
        "Should show repository"
    );

    // Now remove the symlink (simulating unlink command)
    fs::remove_file(&symlink_path)?;
    assert!(!symlink_path.exists(), "Symlink should be removed");

    // Verify which STILL works after unlink (this is the key improvement)
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which").arg(binary_name);
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Which should still work after unlink"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("testuser/testbin"),
        "Should still show repository after unlink: {}",
        stdout
    );
    assert!(
        stdout.contains("2.0.0"),
        "Should still show version after unlink: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_which_multiple_versions() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let binary_name = "multiver";
    let repo = "multiuser/multirepo";

    // Create multiple versions of the same binary
    for version in &["1.0.0", "1.5.0", "2.0.0"] {
        let install_dir = fixture.create_fake_installation(repo, version)?;
        let binary_path = install_dir.join(binary_name);
        fixture.create_executable_with_perms(&binary_path, b"#!/bin/sh\necho 'test binary'")?;
    }

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("which").arg(binary_name);
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "Which should work with multiple versions"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // The command should show the repository
    assert!(
        stdout.contains("multiuser/multirepo"),
        "Should show repository: {}",
        stdout
    );

    // All three installed versions should be reported
    assert!(
        stdout.contains("1.0.0"),
        "Should show version 1.0.0: {}",
        stdout
    );
    assert!(
        stdout.contains("1.5.0"),
        "Should show version 1.5.0: {}",
        stdout
    );
    assert!(
        stdout.contains("2.0.0"),
        "Should show version 2.0.0: {}",
        stdout
    );

    Ok(())
}
