//! Unit tests for error handling and edge cases

use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn test_invalid_repo_formats() -> Result<(), Box<dyn std::error::Error>> {
    // Test various invalid formats
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
