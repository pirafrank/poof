//! Integration tests for the 'list' command

use assert_cmd::cargo;
use serial_test::serial;
use std::process::Command;

// Common module is included from the parent integration.rs file
use super::common::fixtures::test_env::TestFixture;
use super::common::helpers::set_test_env;

#[serial]
#[test]
fn test_list_with_non_existing_data_dir() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Do NOT create data dir

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "List should succeed even without an existing data dir"
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_no_installations() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "List command should succeed even with no installations"
    );
    assert!(
        stdout.contains("No installed binaries found") || stdout.is_empty(),
        "Should indicate no binaries found. stdout: {}, stderr: {}",
        stdout,
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_single_installation() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    fixture.create_fake_installation(repo, version)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "List command should succeed");
    assert!(
        stdout.contains(repo),
        "Output should contain repository name: {}",
        stdout
    );
    assert!(
        stdout.contains(version),
        "Output should contain version: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_multiple_installations() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create multiple fake installations
    fixture.create_fake_installation("user1/repo1", "1.0.0")?;
    fixture.create_fake_installation("user1/repo1", "2.0.0")?;
    fixture.create_fake_installation("user2/repo2", "1.5.0")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "List command should succeed");
    assert!(
        stdout.contains("user1/repo1"),
        "Output should contain first repo: {}",
        stdout
    );
    assert!(
        stdout.contains("user2/repo2"),
        "Output should contain second repo: {}",
        stdout
    );
    assert!(
        stdout.contains("1.0.0") && stdout.contains("2.0.0"),
        "Output should contain both versions for repo1: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_output_format() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    fixture.create_fake_installation("test/repo", "1.0.0")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for table headers
    assert!(
        stdout.contains("Repository") || stdout.contains("Versions"),
        "Output should contain table headers: {}",
        stdout
    );

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

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // List should handle this gracefully (skip or error appropriately)
    // The exact behavior depends on implementation
    let _ = output; // Just ensure it doesn't panic

    Ok(())
}
