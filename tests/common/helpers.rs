//! Helper functions for testing
//! This module provides helper functions for testing
//! such as running commands and checking strings.

/// Helper function to run a command and capture output
#[allow(dead_code)]
pub fn run_command(args: &[&str]) -> Result<(bool, String, String), Box<dyn std::error::Error>> {
    use std::process::Command;

    let output = Command::new(env!("CARGO_BIN_EXE_poof"))
        .args(args)
        .output()?;

    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok((success, stdout, stderr))
}

/// Helper to check if a string contains all required substrings
#[allow(dead_code)]
pub fn assert_contains_all(text: &str, substrings: &[&str]) {
    for substring in substrings {
        assert!(
            text.contains(substring),
            "Expected text to contain '{}', but it didn't. Text was: {}",
            substring,
            text
        );
    }
}