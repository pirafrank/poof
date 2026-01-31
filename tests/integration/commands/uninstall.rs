//! Integration tests for the 'uninstall' command

use assert_cmd::{assert::OutputAssertExt, cargo};
use serial_test::serial;
use std::io::Write;
use std::process::{Command, Stdio};

// Common module is included from the parent integration.rs file
use super::common::fixtures::test_env::TestFixture;
use super::common::repo_format_validation::*;

fn run_uninstall_with_input(
    fixture: &TestFixture,
    args: &[&str],
    input: &[u8],
) -> Result<std::process::Output, Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let mut child = cmd
        .arg("uninstall")
        .args(args)
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_DATA_HOME",
            fixture
                .home_dir
                .join(".local")
                .join("share")
                .to_str()
                .unwrap(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let mut stdin = child.stdin.take().expect("Cannot open stdin");
        stdin.write_all(input)?;
        stdin.flush()?;
    }

    Ok(child.wait_with_output()?)
}

#[serial]
#[test]
fn test_uninstall_requires_repo() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("uninstall").assert().failure();
    Ok(())
}

#[serial]
#[test]
fn test_uninstall_invalid_repo_format() -> Result<(), Box<dyn std::error::Error>> {
    test_invalid_repo_formats_for_command("uninstall")
}

#[serial]
#[test]
fn test_uninstall_requires_version_or_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let output = cmd.arg("uninstall").arg("user/repo").output()?;

    // Should fail because neither --version nor --all is provided
    assert!(
        !output.status.success(),
        "Should fail without --version or --all"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("required") || stderr.contains("version") || stderr.contains("all"),
        "Error should mention missing required flag: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_uninstall_version_not_exists() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let output =
        run_uninstall_with_input(&fixture, &["user/repo", "--version", "1.0.0"], b"yes\n")?;

    assert!(
        output.status.success(),
        "Should succeed when version doesn't exist"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not installed") || stderr.contains("Version"),
        "Should indicate version not installed: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_uninstall_slug_not_exists() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let output = run_uninstall_with_input(&fixture, &["user/repo", "--all"], b"yes\n")?;

    assert!(
        output.status.success(),
        "Should succeed when slug doesn't exist"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("no versions")
            || stderr.to_lowercase().contains("nothing to do"),
        "Should indicate repository not installed: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_uninstall_version_with_confirmation_yes() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    let install_dir = fixture.create_fake_installation(repo, version)?;

    assert!(
        install_dir.exists(),
        "Install directory should exist before uninstall"
    );

    let output = run_uninstall_with_input(&fixture, &[repo, "--version", version], b"yes\n")?;

    assert!(
        output.status.success(),
        "Uninstall should succeed with 'yes' confirmation"
    );

    // Verify the installation was deleted
    assert!(
        !install_dir.exists(),
        "Installation directory should be deleted after confirmation"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.to_lowercase().contains("successfully removed")
            || stderr.to_lowercase().contains("removed"),
        "Should confirm removal: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_uninstall_version_with_confirmation_no() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    let install_dir = fixture.create_fake_installation(repo, version)?;

    assert!(
        install_dir.exists(),
        "Install directory should exist before uninstall"
    );

    let output = run_uninstall_with_input(&fixture, &[repo, "--version", version], b"no\n")?;

    assert!(
        output.status.success(),
        "Uninstall should succeed even when cancelled"
    );

    // Verify the installation still exists
    assert!(
        install_dir.exists(),
        "Installation directory should NOT be deleted after cancellation"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cancelled") || stderr.contains("Uninstall cancelled"),
        "Output should indicate uninstall was cancelled: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_uninstall_all_with_confirmation() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create multiple versions of the same repo
    let repo = "testuser/testrepo";
    let version1 = "1.0.0";
    let version2 = "2.0.0";

    let install_dir1 = fixture.create_fake_installation(repo, version1)?;
    let install_dir2 = fixture.create_fake_installation(repo, version2)?;

    assert!(install_dir1.exists(), "Version 1 should exist");
    assert!(install_dir2.exists(), "Version 2 should exist");

    let output = run_uninstall_with_input(&fixture, &[repo, "--all"], b"yes\n")?;

    assert!(
        output.status.success(),
        "Uninstall --all should succeed with 'yes' confirmation"
    );

    // Verify both versions were deleted
    assert!(
        !install_dir1.exists(),
        "Version 1 should be deleted after uninstall --all"
    );
    assert!(
        !install_dir2.exists(),
        "Version 2 should be deleted after uninstall --all"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("ALL versions") || stderr.contains("all versions"),
        "Warning should mention ALL versions: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_uninstall_with_yes_flag() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    let install_dir = fixture.create_fake_installation(repo, version)?;

    assert!(
        install_dir.exists(),
        "Install directory should exist before uninstall"
    );

    // Use -y flag to skip confirmation
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let output = cmd
        .arg("uninstall")
        .arg(repo)
        .arg("--version")
        .arg(version)
        .arg("--yes")
        .env("HOME", fixture.home_dir.to_str().unwrap())
        .env(
            "XDG_DATA_HOME",
            fixture
                .home_dir
                .join(".local")
                .join("share")
                .to_str()
                .unwrap(),
        )
        .output()?;

    assert!(
        output.status.success(),
        "Uninstall should succeed with -y flag"
    );

    // Verify the installation was deleted
    assert!(
        !install_dir.exists(),
        "Installation should be deleted without prompting"
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_uninstall_cleans_broken_symlinks() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    let binary_name = "testrepo";

    let install_dir = fixture.create_fake_installation(repo, version)?;
    let binary_path = install_dir.join(binary_name);

    // Create a symlink pointing to the binary
    let symlink_path = fixture.bin_dir.join(binary_name);
    std::os::unix::fs::symlink(&binary_path, &symlink_path)?;

    assert!(symlink_path.exists(), "Symlink should exist");
    assert!(binary_path.exists(), "Binary should exist");

    // Uninstall the version (which will delete the binary, breaking the symlink)
    let output = run_uninstall_with_input(&fixture, &[repo, "--version", version], b"yes\n")?;

    assert!(output.status.success(), "Uninstall should succeed");

    // Verify the symlink was cleaned up
    assert!(
        !symlink_path.exists(),
        "Broken symlink should be cleaned up automatically"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("successfully removed"),
        "Output should mention cleaning up broken symlinks: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
#[cfg(not(target_os = "windows"))]
fn test_uninstall_preserves_valid_symlinks() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create two fake installations
    let repo1 = "user1/tool1";
    let repo2 = "user2/tool2";
    let version = "1.0.0";

    let install_dir1 = fixture.create_fake_installation(repo1, version)?;
    let install_dir2 = fixture.create_fake_installation(repo2, version)?;

    // Create symlinks for both
    let binary1_path = install_dir1.join("tool1");
    let binary2_path = install_dir2.join("tool2");
    let symlink1 = fixture.bin_dir.join("tool1");
    let symlink2 = fixture.bin_dir.join("tool2");

    std::os::unix::fs::symlink(&binary1_path, &symlink1)?;
    std::os::unix::fs::symlink(&binary2_path, &symlink2)?;

    assert!(symlink1.exists(), "Symlink 1 should exist");
    assert!(symlink2.exists(), "Symlink 2 should exist");

    // Uninstall repo1 only
    let output = run_uninstall_with_input(&fixture, &[repo1, "--version", version], b"yes\n")?;

    assert!(output.status.success(), "Uninstall should succeed");

    // Verify symlink1 was cleaned (broken), but symlink2 still exists (valid)
    assert!(!symlink1.exists(), "Broken symlink should be cleaned up");
    assert!(symlink2.exists(), "Valid symlink should be preserved");
    assert!(install_dir2.exists(), "Other installation should remain");

    Ok(())
}

#[serial]
#[test]
fn test_uninstall_all_multiple_versions() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create multiple versions
    let repo = "testuser/multitool";
    let versions = ["1.0.0", "1.1.0", "2.0.0", "2.1.0"];

    for version in &versions {
        fixture.create_fake_installation(repo, version)?;
    }

    // Verify all exist
    for version in &versions {
        let install_dir = fixture.get_install_path(repo, version);
        assert!(
            install_dir.exists(),
            "Version {} should exist before uninstall",
            version
        );
    }

    // Uninstall all
    let output = run_uninstall_with_input(&fixture, &[repo, "--all"], b"yes\n")?;

    assert!(output.status.success(), "Uninstall --all should succeed");

    // Verify all versions were deleted
    for version in &versions {
        let install_dir = fixture.get_install_path(repo, version);
        assert!(
            !install_dir.exists(),
            "Version {} should be deleted after uninstall --all",
            version
        );
    }

    // Verify the entire slug directory was removed
    let slug_dir = fixture.data_dir.join("testuser").join("multitool");
    assert!(
        !slug_dir.exists(),
        "Slug directory should be completely removed"
    );

    Ok(())
}

#[serial]
#[test]
fn test_uninstall_case_insensitive_confirmation() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    let install_dir = fixture.create_fake_installation(repo, version)?;

    let output = run_uninstall_with_input(&fixture, &[repo, "--version", version], b"YES\n")?;

    assert!(
        output.status.success(),
        "Uninstall should succeed with uppercase 'YES'"
    );

    // Verify the installation was deleted
    assert!(
        !install_dir.exists(),
        "Installation should be deleted with uppercase confirmation"
    );

    Ok(())
}

#[serial]
#[test]
fn test_uninstall_conflicts_version_and_all() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    let output = cmd
        .arg("uninstall")
        .arg("user/repo")
        .arg("--version")
        .arg("1.0.0")
        .arg("--all")
        .output()?;

    // Should fail because --version and --all conflict
    assert!(
        !output.status.success(),
        "Should fail when both --version and --all are provided"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("conflict") || stderr.contains("cannot be used"),
        "Error should mention conflicting flags: {}",
        stderr
    );

    Ok(())
}
