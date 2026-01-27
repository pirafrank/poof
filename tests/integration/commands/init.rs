//! Integration tests for the 'init' command

use assert_cmd::{assert::OutputAssertExt, cargo};
use predicates::prelude::*;
use std::process::Command;

#[test]
fn test_init_bash() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("--shell")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("export PATH="))
        .stdout(predicate::str::contains("/poof/bin:$PATH"));
    Ok(())
}

#[test]
fn test_init_zsh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("--shell")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("export PATH="))
        .stdout(predicate::str::contains("/poof/bin:$PATH"));
    Ok(())
}

#[test]
fn test_init_fish() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("--shell")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("fish_add_path"))
        .stdout(predicate::str::contains("/poof/bin"));
    Ok(())
}

#[test]
fn test_init_elvish() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("--shell")
        .arg("elvish")
        .assert()
        .success()
        .stdout(predicate::str::contains("set paths ="))
        .stdout(predicate::str::contains("/poof/bin"));
    Ok(())
}

#[test]
fn test_init_powershell() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("--shell")
        .arg("powershell")
        .assert()
        .success()
        .stdout(predicate::str::contains("$env:PATH ="))
        .stdout(predicate::str::contains("/poof/bin"));
    Ok(())
}

#[test]
fn test_init_nushell() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("--shell")
        .arg("nushell")
        .assert()
        .success()
        .stdout(predicate::str::contains("$env.PATH ="))
        .stdout(predicate::str::contains("prepend"))
        .stdout(predicate::str::contains("/poof/bin"));
    Ok(())
}

#[test]
fn test_init_xonsh() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("--shell")
        .arg("xonsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("$PATH.insert"))
        .stdout(predicate::str::contains("/poof/bin"));
    Ok(())
}

#[test]
fn test_init_short_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("-s")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("export PATH="));
    Ok(())
}

#[test]
fn test_init_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Generate shell-specific init script",
        ))
        .stdout(predicate::str::contains("--shell"));
    Ok(())
}

#[test]
fn test_init_missing_shell_arg() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ));
    Ok(())
}

#[test]
fn test_init_invalid_shell() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("init")
        .arg("--shell")
        .arg("invalid_shell")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unsupported shell"));
    Ok(())
}
