//! Integration tests for the 'use' command (make_default)
//! Note: module named make_default because 'use' is a Rust keyword

use assert_cmd::{assert::OutputAssertExt, cargo};
use predicates::prelude::*;
use serial_test::serial;
use std::process::Command;

// Common module is included from the parent integration.rs file
use super::common::fixtures::test_env::TestFixture;
use super::common::helpers::set_test_env;
use super::common::repo_format_validation::*;

#[test]
fn test_use_missing_repo() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
    Ok(())
}

#[serial]
#[test]
fn test_use_without_version_auto_selects_latest() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let repo = "testuser/testrepo";
    fixture.create_fake_installation(repo, "1.0.0")?;
    fixture.create_fake_installation(repo, "2.0.0")?;
    let latest = fixture.create_fake_installation(repo, "3.0.0")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use").arg(repo);
    set_test_env(&mut cmd, &fixture);
    cmd.assert().success().stderr(predicate::str::contains(
        "newest installed version as default",
    ));
    assert_eq!(
        std::fs::read_link(fixture.bin_dir.join("testrepo"))?,
        latest.join("testrepo")
    );
    Ok(())
}

#[serial]
#[test]
fn test_use_without_version_requires_installation() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Try to use a repository that doesn't exist (without version)
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use").arg("nonexistent/repo");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // Should fail because repository is not installed
    assert!(
        !output.status.success(),
        "Use command should fail when repository is not installed"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not installed") || stderr.contains("not found"),
        "Should indicate repository is not installed: {}",
        stderr
    );

    // Should suggest installing the repository
    assert!(
        stderr.contains("poof install") || stderr.contains("Install it using"),
        "Should suggest installing with poof install: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_use_with_nonexistent_version() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let repo = "testuser/testrepo";
    let version1 = "1.0.0";

    // create installation for other version
    let install_dir1 = fixture.create_fake_installation(repo, version1)?;
    // Get binary name
    let binary_name = repo.split('/').next_back().unwrap_or("testrepo");
    // Verify fake binary exists
    assert!(
        install_dir1.join(binary_name).exists(),
        "Version 1 binary should exist"
    );

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use").arg(repo).arg("999.999.999");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(
        !output.status.success(),
        "Use should fail with nonexistent version"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not installed") || stderr.contains("not found"),
        "Should indicate version not found: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_use_sets_default_version() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let repo = "testuser/testrepo";
    let previous = fixture.create_fake_installation(repo, "1.0.0")?;
    let selected = fixture.create_fake_installation(repo, "2.0.0")?;
    fixture.create_bin_symlink("testrepo", &previous.join("testrepo"))?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use").arg(repo).arg("2.0.0");
    set_test_env(&mut cmd, &fixture);
    cmd.assert().success();
    assert_eq!(
        std::fs::read_link(fixture.bin_dir.join("testrepo"))?,
        selected.join("testrepo")
    );
    Ok(())
}

#[test]
fn test_use_comprehensive_invalid_repo_formats() -> Result<(), Box<dyn std::error::Error>> {
    test_invalid_repo_formats_for_command("use")
}

#[test]
fn test_use_comprehensive_valid_repo_formats() -> Result<(), Box<dyn std::error::Error>> {
    test_valid_repo_formats_for_command("use")
}

#[serial]
#[test]
fn test_use_explicit_version_with_multiple_installations() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = TestFixture::new()?;
    let repo = "testuser/testrepo";
    let selected = fixture.create_fake_installation(repo, "1.0.0")?;
    fixture.create_fake_installation(repo, "2.0.0")?;
    fixture.create_fake_installation(repo, "3.0.0")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use").arg(repo).arg("1.0.0");
    set_test_env(&mut cmd, &fixture);
    cmd.assert().success();
    assert_eq!(
        std::fs::read_link(fixture.bin_dir.join("testrepo"))?,
        selected.join("testrepo")
    );
    Ok(())
}

#[serial]
#[test]
fn test_use_empty_installation_preserves_default() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let repo = "testuser/testrepo";
    let previous = fixture
        .create_fake_installation(repo, "1.0.0")?
        .join("testrepo");
    fixture.create_bin_symlink("testrepo", &previous)?;
    std::fs::create_dir_all(fixture.get_install_path(repo, "2.0.0"))?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use").arg(repo).arg("2.0.0");
    set_test_env(&mut cmd, &fixture);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("No binaries found").and(predicate::str::contains(
                "Consider reinstalling the version",
            )),
        );
    assert_eq!(
        std::fs::read_link(fixture.bin_dir.join("testrepo"))?,
        previous
    );
    Ok(())
}

#[serial]
#[test]
fn test_use_non_executable_installation_preserves_default() -> Result<(), Box<dyn std::error::Error>>
{
    use std::os::unix::fs::PermissionsExt;

    let fixture = TestFixture::new()?;
    let repo = "testuser/testrepo";
    let previous = fixture
        .create_fake_installation(repo, "1.0.0")?
        .join("testrepo");
    fixture.create_bin_symlink("testrepo", &previous)?;
    let selected = fixture
        .create_fake_installation(repo, "2.0.0")?
        .join("testrepo");
    std::fs::set_permissions(selected, std::fs::Permissions::from_mode(0o644))?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use").arg(repo).arg("2.0.0");
    set_test_env(&mut cmd, &fixture);
    cmd.assert()
        .failure()
        .stderr(
            predicate::str::contains("No binaries found").and(predicate::str::contains(
                "Consider reinstalling the version",
            )),
        );
    assert_eq!(
        std::fs::read_link(fixture.bin_dir.join("testrepo"))?,
        previous
    );
    Ok(())
}

#[serial]
#[test]
fn test_use_links_only_usable_files_including_nested_binaries(
) -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    let fixture = TestFixture::new()?;
    let repo = "testuser/testrepo";
    let install = fixture.create_fake_installation(repo, "1.0.0")?;
    let nested_dir = install.join("nested");
    std::fs::create_dir_all(&nested_dir)?;
    // A native executable alongside the top-level shebang script.
    let nested_binary = nested_dir.join("helper");
    std::fs::copy(cargo::cargo_bin!("poof"), &nested_binary)?;
    std::fs::set_permissions(&nested_binary, std::fs::Permissions::from_mode(0o755))?;
    let disabled = install.join("disabled");
    std::fs::write(&disabled, b"#!/bin/sh\necho disabled\n")?;
    std::fs::set_permissions(disabled, std::fs::Permissions::from_mode(0o644))?;
    fixture.create_executable_with_perms(&install.join("README"), b"Not an executable\n")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use").arg(repo).arg("1.0.0");
    set_test_env(&mut cmd, &fixture);
    cmd.assert().success();
    assert_eq!(
        std::fs::read_link(fixture.bin_dir.join("testrepo"))?,
        install.join("testrepo")
    );
    assert_eq!(
        std::fs::read_link(fixture.bin_dir.join("helper"))?,
        nested_binary
    );
    assert!(std::fs::symlink_metadata(fixture.bin_dir.join("disabled")).is_err());
    assert!(std::fs::symlink_metadata(fixture.bin_dir.join("README")).is_err());
    assert_eq!(std::fs::read_dir(&fixture.bin_dir)?.count(), 2);
    Ok(())
}

#[serial]
#[test]
fn test_use_empty_latest_installation_preserves_default() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = TestFixture::new()?;
    let repo = "testuser/testrepo";
    let previous = fixture
        .create_fake_installation(repo, "1.0.0")?
        .join("testrepo");
    fixture.create_bin_symlink("testrepo", &previous)?;
    std::fs::create_dir_all(fixture.get_install_path(repo, "2.0.0"))?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("use").arg(repo);
    set_test_env(&mut cmd, &fixture);
    cmd.assert().failure().stderr(
        predicate::str::contains("No binaries found to set as default for version 2.0.0").and(
            predicate::str::contains("Consider reinstalling the version"),
        ),
    );
    assert_eq!(
        std::fs::read_link(fixture.bin_dir.join("testrepo"))?,
        previous
    );
    Ok(())
}
