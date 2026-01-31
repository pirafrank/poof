//! Helper functions for testing
//! This module provides helper functions for testing
//! such as running commands and checking strings.

use super::fixtures::test_env::TestFixture;
use std::process::Command;

/// Helper to set environment variables from TestFixture on a Command
/// This ensures tests run in an isolated environment without touching the real filesystem
pub fn set_test_env(cmd: &mut Command, fixture: &TestFixture) {
    let (home_key, home_val) = fixture.env_home();
    cmd.env(home_key, home_val);

    #[cfg(target_os = "linux")]
    {
        if let Some((data_key, data_val)) = fixture.env_data_home() {
            cmd.env(data_key, data_val);
        }
        if let Some((cache_key, cache_val)) = fixture.env_cache_home() {
            cmd.env(cache_key, cache_val);
        }
    }
}

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
