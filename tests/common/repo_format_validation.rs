//! Common repository format validation utilities for testing
//!
//! These functions can be used to test repository format validation
//! across different commands (install, update, download, etc.)

use assert_cmd::cargo;
use std::process::Command;

/// Test that various invalid repository formats are rejected by a command
///
/// # Arguments
/// * `command` - The poof subcommand to test (e.g., "install", "update", "download")
///
/// # Example
/// ```no_run
/// test_invalid_repo_formats_for_command("install")?;
/// ```
pub fn test_invalid_repo_formats_for_command(
    command: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let invalid_formats = vec![
        "invalid-repo-format",
        "user/repo/extra",
        "user",
        "/repo",
        "user/",
        "user repo",
        "user@repo",
        "user#repo",
    ];

    for invalid in invalid_formats {
        let mut cmd = Command::new(cargo::cargo_bin!("poof"));
        let output = cmd.arg(command).arg(invalid).output()?;
        assert!(
            !output.status.success(),
            "Command '{}' should reject format '{}', but it didn't",
            command,
            invalid
        );
    }

    Ok(())
}

/// Test that valid repository formats are accepted by a command
///
/// Note: This only tests that the format is accepted, not that the command succeeds.
/// The command may still fail due to network issues, missing repositories, etc.
///
/// # Arguments
/// * `command` - The poof subcommand to test (e.g., "install", "update", "download")
///
/// # Example
/// ```no_run
/// test_valid_repo_formats_for_command("install")?;
/// ```
pub fn test_valid_repo_formats_for_command(
    command: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let valid_formats = vec![
        "user/repo",
        "user-name/repo-name",
        "user_name/repo_name",
        "user123/repo123",
        "user/repo-name",
    ];

    for valid in valid_formats {
        let mut cmd = Command::new(cargo::cargo_bin!("poof"));
        let output = cmd.arg(command).arg(valid).output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should not fail with format error
        assert!(
            !stderr.contains("Repository must be in the format"),
            "Command '{}' should accept format '{}', but got format error: {}",
            command,
            valid,
            stderr
        );
    }

    Ok(())
}
