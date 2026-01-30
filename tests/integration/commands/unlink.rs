//! Integration tests for the 'unlink' command

use assert_cmd::{assert::OutputAssertExt, cargo};
use serial_test::serial;
use std::io::Write;
use std::process::{Command, Stdio};

// Common module is included from the parent integration.rs file
use super::common::fixtures::test_env::TestFixture;

#[cfg(not(target_os = "windows"))]
fn setup_test_symlink_env(
    fixture: &TestFixture,
    symlink_name: &str,
) -> std::io::Result<std::path::PathBuf> {
    use std::fs;
    let target = fixture.bin_dir.join("target_binary");
    fs::write(&target, b"content")?;
    let symlink_path = fixture.bin_dir.join(symlink_name);
    std::os::unix::fs::symlink(&target, &symlink_path)?;
    Ok(symlink_path)
}

fn run_unlink_with_input(
    fixture: &TestFixture,
    binary_name: &str,
    input: &[u8],
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let mut child = cmd
        .arg("unlink")
        .arg(binary_name)
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
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let mut stdin = child.stdin.take().expect("Cannot open stdin");
        stdin.write_all(input)?;
        stdin.flush()?;
    }

    Ok(child.wait_with_output()?)
}

#[serial]
#[test]
fn test_unlink_requires_binary_name() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("unlink").assert().failure();
    Ok(())
}

#[serial]
#[test]
fn test_unlink_nonexistent_binary() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let output = run_unlink_with_input(&fixture, "nonexistent_binary", b"yes\n")?;

    assert!(
        output.status.success(),
        "Should succeed when binary doesn't exist"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No binary named") || stderr.contains("nonexistent_binary"),
        "Should indicate binary not found: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_unlink_with_confirmation_yes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let symlink_name = "test_binary";
    let symlink_path = setup_test_symlink_env(&fixture, symlink_name)?;

    assert!(symlink_path.exists(), "Symlink should exist before unlink");

    let output = run_unlink_with_input(&fixture, symlink_name, b"yes\n")?;

    assert!(
        output.status.success(),
        "Unlink should succeed with 'yes' confirmation"
    );

    // Verify the symlink was deleted
    assert!(
        !symlink_path.exists(),
        "Symlink should be deleted after confirmation"
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_unlink_with_confirmation_y() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let symlink_name = "test_binary";
    let symlink_path = setup_test_symlink_env(&fixture, symlink_name)?;

    assert!(symlink_path.exists(), "Symlink should exist before unlink");

    let output = run_unlink_with_input(&fixture, symlink_name, b"y\n")?;

    assert!(
        output.status.success(),
        "Unlink should succeed with 'y' confirmation"
    );

    // Verify the symlink was deleted
    assert!(
        !symlink_path.exists(),
        "Symlink should be deleted after confirmation with 'y'"
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_unlink_with_confirmation_no() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let symlink_name = "test_binary";
    let symlink_path = setup_test_symlink_env(&fixture, symlink_name)?;

    assert!(symlink_path.exists(), "Symlink should exist before unlink");

    let output = run_unlink_with_input(&fixture, symlink_name, b"no\n")?;

    assert!(
        output.status.success(),
        "Unlink should succeed even when cancelled"
    );

    // Verify the symlink still exists
    assert!(
        symlink_path.exists(),
        "Symlink should NOT be deleted after cancellation with 'no'"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cancelled") || stderr.contains("Unlink cancelled"),
        "Output should indicate unlink was cancelled: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_unlink_with_confirmation_n() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let symlink_name = "test_binary";
    let symlink_path = setup_test_symlink_env(&fixture, symlink_name)?;

    assert!(symlink_path.exists(), "Symlink should exist before unlink");

    let output = run_unlink_with_input(&fixture, symlink_name, b"n\n")?;

    assert!(
        output.status.success(),
        "Unlink should succeed even when cancelled"
    );

    // Verify the symlink still exists
    assert!(
        symlink_path.exists(),
        "Symlink should NOT be deleted after cancellation with 'n'"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cancelled") || stderr.contains("Unlink cancelled"),
        "Output should indicate unlink was cancelled: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_unlink_with_yes_flag() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let fixture = TestFixture::new()?;

    // Create a fake target and symlink
    let target = fixture.bin_dir.join("target_binary");
    fs::write(&target, b"content")?;

    let symlink_name = "test_binary";
    let symlink_path = fixture.bin_dir.join(symlink_name);
    std::os::unix::fs::symlink(&target, &symlink_path)?;

    assert!(symlink_path.exists(), "Symlink should exist before unlink");

    // Use -y flag to skip confirmation
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let output = cmd
        .arg("unlink")
        .arg(symlink_name)
        .arg("-y")
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

    assert!(
        output.status.success(),
        "Unlink should succeed with -y flag"
    );

    // Verify the symlink was deleted
    assert!(
        !symlink_path.exists(),
        "Symlink should be deleted without prompting"
    );

    Ok(())
}

#[serial]
#[test]
fn test_unlink_regular_file_not_symlink() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let fixture = TestFixture::new()?;

    // Create a regular file (not a symlink)
    let binary_name = "regular_file";
    let file_path = fixture.bin_dir.join(binary_name);
    fs::write(&file_path, b"#!/bin/sh\necho 'regular file'")?;

    assert!(file_path.exists(), "File should exist before unlink");

    let output = run_unlink_with_input(&fixture, binary_name, b"yes\n")?;

    assert!(
        !output.status.success(),
        "Unlink should fail for regular files"
    );

    // Verify the file still exists (wasn't deleted)
    assert!(
        file_path.exists(),
        "Regular file should NOT be deleted (safety check)"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not a symlink") || stderr.contains("Refusing to delete"),
        "Error should mention it's not a symlink: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_unlink_case_insensitive_confirmation() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;

    let fixture = TestFixture::new()?;

    // Create a fake target and symlink
    let target = fixture.bin_dir.join("target_binary");
    fs::write(&target, b"content")?;

    let symlink_name = "test_binary";
    let symlink_path = fixture.bin_dir.join(symlink_name);
    std::os::unix::fs::symlink(&target, &symlink_path)?;

    let output = run_unlink_with_input(&fixture, symlink_name, b"YES\n")?;

    assert!(
        output.status.success(),
        "Unlink should succeed with uppercase 'YES'"
    );

    // Verify the symlink was deleted
    assert!(
        !symlink_path.exists(),
        "Symlink should be deleted with uppercase confirmation"
    );

    Ok(())
}
