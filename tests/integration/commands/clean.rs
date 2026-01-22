//! Integration tests for the 'clean' command

use assert_cmd::cargo;
use serial_test::serial;
use std::io::Write;
use std::process::{Command, Stdio};

// Common module is included from the parent integration.rs file
use super::common::fixtures::test_env::TestFixture;

#[serial]
#[test]
fn test_clean_when_cache_dir_not_exists() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Remove the cache directory if it was created
    if fixture.cache_dir.exists() {
        std::fs::remove_dir_all(&fixture.cache_dir)?;
    }

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let output = cmd
        .arg("clean")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_CACHE_HOME",
            fixture.home_dir.join(".cache").to_str().unwrap(),
        )
        .output()?;

    assert!(
        output.status.success(),
        "Clean should succeed even when cache directory doesn't exist"
    );

    Ok(())
}

#[serial]
#[test]
fn test_clean_with_confirmation_yes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create cache directory with some content
    std::fs::create_dir_all(&fixture.cache_dir)?;
    std::fs::write(fixture.cache_dir.join("test_file.txt"), b"test content")?;

    assert!(
        fixture.cache_dir.exists(),
        "Cache dir should exist before clean"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let mut child = cmd
        .arg("clean")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_CACHE_HOME",
            fixture.cache_dir.parent().unwrap().to_str().unwrap(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write to stdin and explicitly drop it to signal EOF
    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(b"yes\n")?;
        stdin.flush()?;
    } // stdin is dropped here, signaling EOF

    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "Clean command should succeed with 'yes' confirmation"
    );

    // Verify the cache directory was deleted
    assert!(
        !fixture.cache_dir.exists(),
        "Cache directory should be deleted after confirmation"
    );

    Ok(())
}

#[serial]
#[test]
fn test_clean_with_confirmation_y() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create cache directory with some content
    std::fs::create_dir_all(&fixture.cache_dir)?;
    std::fs::write(fixture.cache_dir.join("test_file.txt"), b"test content")?;

    assert!(
        fixture.cache_dir.exists(),
        "Cache dir should exist before clean"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let mut child = cmd
        .arg("clean")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_CACHE_HOME",
            fixture.cache_dir.parent().unwrap().to_str().unwrap(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write to stdin and explicitly drop it to signal EOF
    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(b"y\n")?;
        stdin.flush()?;
    } // stdin is dropped here, signaling EOF

    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "Clean command should succeed with 'y' confirmation"
    );

    // Verify the cache directory was deleted
    assert!(
        !fixture.cache_dir.exists(),
        "Cache directory should be deleted after confirmation with 'y'"
    );

    Ok(())
}

#[serial]
#[test]
fn test_clean_with_confirmation_no() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create cache directory with some content
    std::fs::create_dir_all(&fixture.cache_dir)?;
    std::fs::write(fixture.cache_dir.join("test_file.txt"), b"test content")?;

    assert!(
        fixture.cache_dir.exists(),
        "Cache dir should exist before clean"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let mut child = cmd
        .arg("clean")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_CACHE_HOME",
            fixture.cache_dir.parent().unwrap().to_str().unwrap(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write to stdin and explicitly drop it to signal EOF
    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(b"n\n")?;
        stdin.flush()?;
    } // stdin is dropped here, signaling EOF

    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "Clean command should succeed even when cancelled"
    );

    // Verify the cache directory still exists
    assert!(
        fixture.cache_dir.exists(),
        "Cache directory should NOT be deleted after cancellation with 'n'"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cancelled") || stderr.contains("Cleanup cancelled"),
        "Output should indicate cleanup was cancelled: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_clean_with_confirmation_empty() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create cache directory with some content
    std::fs::create_dir_all(&fixture.cache_dir)?;
    std::fs::write(fixture.cache_dir.join("test_file.txt"), b"test content")?;

    assert!(
        fixture.cache_dir.exists(),
        "Cache dir should exist before clean"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let mut child = cmd
        .arg("clean")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_CACHE_HOME",
            fixture.cache_dir.parent().unwrap().to_str().unwrap(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write to stdin and explicitly drop it to signal EOF
    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(b"\n")?;
        stdin.flush()?;
    } // stdin is dropped here, signaling EOF

    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "Clean command should succeed with empty input"
    );

    // Verify the cache directory still exists
    assert!(
        fixture.cache_dir.exists(),
        "Cache directory should NOT be deleted with empty input"
    );

    Ok(())
}

#[serial]
#[test]
fn test_clean_with_confirmation_other_input() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create cache directory with some content
    std::fs::create_dir_all(&fixture.cache_dir)?;
    std::fs::write(fixture.cache_dir.join("test_file.txt"), b"test content")?;

    assert!(
        fixture.cache_dir.exists(),
        "Cache dir should exist before clean"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let mut child = cmd
        .arg("clean")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_CACHE_HOME",
            fixture.cache_dir.parent().unwrap().to_str().unwrap(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write to stdin and explicitly drop it to signal EOF
    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(b"maybe\n")?;
        stdin.flush()?;
    } // stdin is dropped here, signaling EOF

    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "Clean command should succeed with invalid input"
    );

    // Verify the cache directory still exists
    assert!(
        fixture.cache_dir.exists(),
        "Cache directory should NOT be deleted with invalid input"
    );

    Ok(())
}

#[serial]
#[test]
fn test_clean_case_insensitive_confirmation() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create cache directory with some content
    std::fs::create_dir_all(&fixture.cache_dir)?;
    std::fs::write(fixture.cache_dir.join("test_file.txt"), b"test content")?;

    assert!(
        fixture.cache_dir.exists(),
        "Cache dir should exist before clean"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let mut child = cmd
        .arg("clean")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_CACHE_HOME",
            fixture.cache_dir.parent().unwrap().to_str().unwrap(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write to stdin and explicitly drop it to signal EOF
    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(b"YES\n")?;
        stdin.flush()?;
    } // stdin is dropped here, signaling EOF

    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "Clean command should succeed with 'YES' confirmation"
    );

    // Verify the cache directory was deleted
    assert!(
        !fixture.cache_dir.exists(),
        "Cache directory should be deleted with uppercase 'YES'"
    );

    Ok(())
}

#[serial]
#[test]
fn test_clean_case_insensitive_y() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create cache directory with some content
    std::fs::create_dir_all(&fixture.cache_dir)?;
    std::fs::write(fixture.cache_dir.join("test_file.txt"), b"test content")?;

    assert!(
        fixture.cache_dir.exists(),
        "Cache dir should exist before clean"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let mut child = cmd
        .arg("clean")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_CACHE_HOME",
            fixture.cache_dir.parent().unwrap().to_str().unwrap(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write to stdin and explicitly drop it to signal EOF
    {
        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        stdin.write_all(b"Y\n")?;
        stdin.flush()?;
    } // stdin is dropped here, signaling EOF

    let output = child.wait_with_output()?;

    assert!(
        output.status.success(),
        "Clean command should succeed with 'Y' confirmation"
    );

    // Verify the cache directory was deleted
    assert!(
        !fixture.cache_dir.exists(),
        "Cache directory should be deleted with uppercase 'Y'"
    );

    Ok(())
}
