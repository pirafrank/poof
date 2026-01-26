//! Unit tests for the 'completions' command

use assert_cmd::{assert::OutputAssertExt, cargo};
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_completions_bash() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("completions")
        .arg("--shell")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"))
        .stdout(predicate::str::contains("poof"));
    Ok(())
}

#[test]
fn test_completions_zsh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("completions")
        .arg("--shell")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef"))
        .stdout(predicate::str::contains("poof"));
    Ok(())
}

#[test]
fn test_completions_fish() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("completions")
        .arg("--shell")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"))
        .stdout(predicate::str::contains("poof"));
    Ok(())
}

#[test]
fn test_completions_short_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("completions")
        .arg("-s")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("complete"));
    Ok(())
}

#[test]
fn test_completions_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("completions")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate shell completions"))
        .stdout(predicate::str::contains("--shell"));
    Ok(())
}
