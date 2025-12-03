//! Unit tests for error handling and edge cases

use assert_cmd::{assert::OutputAssertExt, cargo};
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_invalid_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("nonexistent")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
    Ok(())
}
