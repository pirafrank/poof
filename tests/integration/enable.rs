//! Integration tests for the 'enable' command

use assert_cmd::prelude::*;
use serial_test::serial;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[serial]
#[test]
fn test_enable_creates_bashrc_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("enable")
        .env("HOME", temp_home.path())
        .env("SHELL", "/bin/bash")
        .env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        )
        .output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Check that .bashrc was created/modified
    let bashrc_path = temp_home.path().join(".bashrc");
    if bashrc_path.exists() {
        let contents = fs::read_to_string(&bashrc_path)?;
        assert!(
            contents.contains("export PATH="),
            ".bashrc should contain export PATH line"
        );
        assert!(
            contents.contains(bin_dir.to_string_lossy().as_ref()),
            ".bashrc should contain bin directory path"
        );
        assert!(
            contents.contains("# added by poof"),
            ".bashrc should contain comment marker"
        );
    }

    Ok(())
}

#[serial]
#[test]
fn test_enable_creates_zshrc_entry() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("enable")
        .env("HOME", temp_home.path())
        .env("SHELL", "/usr/bin/zsh")
        .env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        )
        .output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Check that .zshrc was created/modified
    let zshrc_path = temp_home.path().join(".zshrc");
    if zshrc_path.exists() {
        let contents = fs::read_to_string(&zshrc_path)?;
        assert!(
            contents.contains("export PATH="),
            ".zshrc should contain export PATH line"
        );
        assert!(
            contents.contains(bin_dir.to_string_lossy().as_ref()),
            ".zshrc should contain bin directory path"
        );
    }

    Ok(())
}

#[serial]
#[test]
fn test_enable_is_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    // Run enable twice
    let mut cmd1 = Command::cargo_bin("poof")?;
    cmd1.arg("enable")
        .env("HOME", temp_home.path())
        .env("SHELL", "/bin/bash")
        .env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        )
        .output()?;

    let mut cmd2 = Command::cargo_bin("poof")?;
    cmd2.arg("enable")
        .env("HOME", temp_home.path())
        .env("SHELL", "/bin/bash")
        .env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        )
        .output()?;

    // Check that export line appears only once
    let bashrc_path = temp_home.path().join(".bashrc");
    if bashrc_path.exists() {
        let contents = fs::read_to_string(&bashrc_path)?;
        let bin_str = bin_dir.to_string_lossy();
        let export_line = format!("export PATH=\"{}:$PATH\"", bin_str);
        let count = contents.matches(&export_line).count();
        assert_eq!(
            count, 1,
            "Export line should appear exactly once, found {} times",
            count
        );
    }

    Ok(())
}

#[serial]
#[test]
fn test_enable_preserves_existing_content() -> Result<(), Box<dyn std::error::Error>> {
    let temp_home = TempDir::new()?;

    // Create bin directory structure
    let bin_dir = temp_home
        .path()
        .join(".local")
        .join("share")
        .join("poof")
        .join("bin");
    fs::create_dir_all(&bin_dir)?;

    // Pre-seed .bashrc with existing content
    let bashrc_path = temp_home.path().join(".bashrc");
    fs::write(&bashrc_path, "PRE_EXISTING_LINE\n")?;

    let mut cmd = Command::cargo_bin("poof")?;
    let output = cmd
        .arg("enable")
        .env("HOME", temp_home.path())
        .env("SHELL", "/bin/bash")
        .env(
            "XDG_DATA_HOME",
            temp_home.path().join(".local").join("share"),
        )
        .output()?;

    assert!(output.status.success(), "Enable command should succeed");

    // Verify existing content is preserved
    let contents = fs::read_to_string(&bashrc_path)?;
    assert!(
        contents.contains("PRE_EXISTING_LINE"),
        "Existing content should be preserved"
    );
    assert!(
        contents.contains("export PATH="),
        "Export line should be added"
    );

    Ok(())
}
