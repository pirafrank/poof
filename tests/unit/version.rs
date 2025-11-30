//! Unit tests for the 'version' command

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_version_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")))
        .stdout(predicate::str::contains(env!("GIT_COMMIT_HASH")))
        .stdout(predicate::str::contains(env!("BUILD_DATE")))
        .stdout(predicate::str::contains(env!("C_LIB")))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_upper_v_version_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")))
        .stdout(predicate::str::contains(env!("GIT_COMMIT_HASH")))
        .stdout(predicate::str::contains(env!("BUILD_DATE")))
        .stdout(predicate::str::contains(env!("C_LIB")))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_version_command_exists() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")))
        .stdout(predicate::str::contains(env!("GIT_COMMIT_HASH")))
        .stdout(predicate::str::contains(env!("BUILD_DATE")))
        .stdout(predicate::str::contains(env!("C_LIB")))
        .stderr(predicate::str::is_empty());
    Ok(())
}

#[test]
fn test_version_command_with_extra_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("poof")?;
    // Version command should ignore extra args or fail gracefully
    cmd.arg("version").arg("extra").assert().failure(); // clap should reject extra positional args
    Ok(())
}
