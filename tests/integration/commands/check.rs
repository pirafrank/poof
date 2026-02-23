//! Integration tests for the 'check' command

use assert_cmd::{assert::OutputAssertExt, cargo};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_check_not_in_path_exits_2() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let home = temp_dir.path();
    let xdg_data_home = temp_dir.path().join(".local").join("share");
    let path = "/usr/bin:/bin"; // PATH without poof's bin dir

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let output = cmd
        .arg("check")
        .env("HOME", home)
        .env("XDG_DATA_HOME", &xdg_data_home)
        .env("PATH", path)
        .output()?;

    assert_eq!(
        output.status.code(),
        Some(2),
        "Should exit with code 2 when bin directory is not in PATH"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not found in PATH"),
        "Should warn that bin directory is not found in PATH: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_check_not_first_in_path_exits_1() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let home = temp_dir.path();
    let xdg_data_home = temp_dir.path().join(".local").join("share");
    // Construct a PATH where poof's bin dir is present but not first
    #[cfg(target_os = "linux")]
    let bin_dir = xdg_data_home.join("poof").join("bin");
    #[cfg(target_os = "macos")]
    let bin_dir = home
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    let path = format!("/usr/bin:/bin:{}", bin_dir.display());

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let output = cmd
        .arg("check")
        .env("HOME", home)
        .env("XDG_DATA_HOME", &xdg_data_home)
        .env("PATH", &path)
        .output()?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "Should exit with code 1 when bin directory is in PATH but not first"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not the first in PATH") || stderr.contains("not the first"),
        "Should warn that bin directory is not the first in PATH: {}",
        stderr
    );

    Ok(())
}

#[test]
fn test_check_bin_first_in_path_exits_0() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let home = temp_dir.path();
    let xdg_data_home = temp_dir.path().join(".local").join("share");
    // Construct a PATH where poof's bin dir is first
    #[cfg(target_os = "linux")]
    let bin_dir = xdg_data_home.join("poof").join("bin");
    #[cfg(target_os = "macos")]
    let bin_dir = home
        .join("Library")
        .join("Application Support")
        .join("poof")
        .join("bin");
    let path = format!("{}:/usr/bin:/bin", bin_dir.display());

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("check")
        .env("HOME", home)
        .env("XDG_DATA_HOME", &xdg_data_home)
        .env("PATH", &path)
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_check_command_with_extra_args() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    // Check command should ignore extra args or fail gracefully
    cmd.arg("check").arg("extra").assert().failure(); // clap should reject extra positional args
    Ok(())
}
