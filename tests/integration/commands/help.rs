//! Unit tests for the 'help' command

use assert_cmd::{assert::OutputAssertExt, cargo};
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_help_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("poof"))
        .stdout(predicate::str::contains("Commands:"));
    Ok(())
}

#[test]
fn test_help_for_subcommand() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("install")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("install"));
    Ok(())
}
