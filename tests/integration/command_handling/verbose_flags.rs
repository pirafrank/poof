//! Unit tests for clap command parsing

use assert_cmd::{assert::OutputAssertExt, cargo};
use std::process::Command;

#[test]
fn test_verbose_flag() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    // Verbose flag should be accepted
    cmd.arg("-v").arg("version").assert().success();
    Ok(())
}

#[test]
fn test_multiple_verbose_flags() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    // Multiple verbose flags should increase verbosity
    cmd.arg("-vv").arg("version").assert().success();
    Ok(())
}
