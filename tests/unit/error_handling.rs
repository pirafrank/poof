//! Unit tests for error handling and edge cases

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_invalid_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
    Ok(())
}

#[test]
fn test_invalid_repo_format() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("install")
        .arg("invalid-repo-format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Repository must be in the format"));
    Ok(())
}

#[test]
fn test_install_missing_repo() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("install")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
    Ok(())
}

#[test]
fn test_update_missing_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("update").assert().failure();
    Ok(())
}

#[test]
fn test_use_missing_repo() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("use")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
    Ok(())
}

#[test]
fn test_repo_format_with_special_characters() -> Result<(), Box<dyn std::error::Error>> {
    // Test various invalid formats
    let invalid_formats = vec![
        "user/repo/extra",
        "user",
        "/repo",
        "user/",
        "user repo",
        "user@repo",
        "user#repo",
    ];

    for invalid in invalid_formats {
        let mut cmd = Command::cargo_bin("poof")?;
        let output = cmd.arg("install").arg(invalid).output()?;
        assert!(
            !output.status.success(),
            "Format '{}' should be rejected",
            invalid
        );
    }

    Ok(())
}

#[test]
fn test_valid_repo_formats() -> Result<(), Box<dyn std::error::Error>> {
    // These should parse correctly (though they may fail later due to network/other issues)
    let valid_formats = vec![
        "user/repo",
        "user-name/repo-name",
        "user_name/repo_name",
        "user123/repo123",
        "user/repo-name",
    ];

    for valid in valid_formats {
        let mut cmd = Command::cargo_bin("poof")?;
        // We don't check success here since it will fail on network/actual install
        // but we check that it doesn't fail on format validation
        let output = cmd.arg("install").arg(valid).output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Should not fail with format error
        assert!(
            !stderr.contains("Repository must be in the format"),
            "Format '{}' should be accepted, but got: {}",
            valid,
            stderr
        );
    }

    Ok(())
}

#[test]
fn test_update_conflicting_flags() -> Result<(), Box<dyn std::error::Error>> {
    // Note: --all and --self don't actually conflict in the current implementation
    // They both can be used together, though --all takes precedence
    // This test verifies the command handles both flags gracefully
    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("update")
        .arg("--all")
        .arg("--self")
        .output()?;
    
    // Command should succeed (--all takes precedence when both are present)
    assert!(
        output.status.success(),
        "Command should handle both flags (--all takes precedence)"
    );
    
    Ok(())
}

#[test]
fn test_update_all_and_repo_conflict() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("update")
        .arg("user/repo")
        .arg("--all")
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used"));
    Ok(())
}
