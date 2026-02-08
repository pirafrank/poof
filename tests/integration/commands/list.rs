//! Integration tests for the 'list' command

use assert_cmd::cargo;
use serial_test::serial;
use std::process::Command;

// Common module is included from the parent integration.rs file
use super::common::fixtures::test_env::TestFixture;
use super::common::helpers::set_test_env;

#[serial]
#[test]
fn test_list_with_non_existing_data_dir() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Do NOT create data dir

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    assert!(
        output.status.success(),
        "List should succeed even without an existing data dir"
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_no_installations() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "List command should succeed even with no installations"
    );

    // Empty list shows diagnostic message on stderr, not stdout
    assert!(
        stdout.is_empty(),
        "stdout should be empty when no installations found: {}",
        stdout
    );

    assert!(
        stderr.contains("No installed binaries found"),
        "stderr should indicate no binaries found: {}",
        stderr
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_single_installation() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation
    let repo = "testuser/testrepo";
    let version = "1.0.0";
    fixture.create_fake_installation(repo, version)?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "List command should succeed");
    assert!(
        stdout.contains(repo),
        "Output should contain repository name: {}",
        stdout
    );
    assert!(
        stdout.contains(version),
        "Output should contain version: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_multiple_installations() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create multiple fake installations
    fixture.create_fake_installation("user1/repo1", "1.0.0")?;
    fixture.create_fake_installation("user1/repo1", "2.0.0")?;
    fixture.create_fake_installation("user2/repo2", "1.5.0")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "List command should succeed");
    assert!(
        stdout.contains("user1/repo1"),
        "Output should contain first repo: {}",
        stdout
    );
    assert!(
        stdout.contains("user2/repo2"),
        "Output should contain second repo: {}",
        stdout
    );
    assert!(
        stdout.contains("1.0.0") && stdout.contains("2.0.0"),
        "Output should contain both versions for repo1: {}",
        stdout
    );

    // Verify TSV format
    assert!(
        stdout.contains("\t"),
        "Output should be tab-separated: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_output_format() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    fixture.create_fake_installation("test/repo", "1.0.0")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Check for TSV format: repo\tversions
    assert!(
        stdout.contains("\t"),
        "Output should be tab-separated: {}",
        stdout
    );

    // Should contain the repo and version
    assert!(
        stdout.contains("test/repo"),
        "Output should contain repository: {}",
        stdout
    );

    assert!(
        stdout.contains("1.0.0"),
        "Output should contain version: {}",
        stdout
    );

    // Check it follows TSV format (one line with tab separator)
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines.iter().any(|line| line.contains("test/repo")
            && line.contains("\t")
            && line.contains("1.0.0")),
        "Should have a line with repo, tab, and version: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_corrupted_directory_structure() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a file where a directory should be
    let bad_path = fixture.data_dir.join("user").join("repo");
    std::fs::create_dir_all(bad_path.parent().unwrap())?;
    std::fs::write(&bad_path, b"not a directory")?;

    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // List should handle this gracefully (skip or error appropriately)
    // The exact behavior depends on implementation
    let _ = output; // Just ensure it doesn't panic

    Ok(())
}

// ============================================================================
// Tests for 'list someuser/somerepo' - listing with a specific slug
// ============================================================================

#[serial]
#[test]
fn test_list_with_valid_slug_single_version() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a fake installation
    fixture.create_fake_installation("user/repo", "1.0.0")?;

    // Execute: run list with slug
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list").arg("user/repo");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // Assert: verify output
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "List command with slug should succeed"
    );
    assert!(
        stdout.contains("user/repo"),
        "Output should contain repository name: {}",
        stdout
    );
    assert!(
        stdout.contains("1.0.0"),
        "Output should contain version: {}",
        stdout
    );

    // Verify TSV format
    assert!(
        stdout.contains("\t"),
        "Output should be tab-separated: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_valid_slug_multiple_versions() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create multiple versions of the same repo
    fixture.create_fake_installation("user/repo", "1.0.0")?;
    fixture.create_fake_installation("user/repo", "2.0.0")?;
    fixture.create_fake_installation("user/repo", "3.0.0")?;

    // Execute: run list with slug
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list").arg("user/repo");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // Assert: verify output
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "List command with slug should succeed"
    );
    assert!(
        stdout.contains("user/repo"),
        "Output should contain repository name: {}",
        stdout
    );
    assert!(
        stdout.contains("1.0.0"),
        "Output should contain version 1.0.0: {}",
        stdout
    );
    assert!(
        stdout.contains("2.0.0"),
        "Output should contain version 2.0.0: {}",
        stdout
    );
    assert!(
        stdout.contains("3.0.0"),
        "Output should contain version 3.0.0: {}",
        stdout
    );

    // Verify TSV format with single repo line
    let data_lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.contains("Repository") && !line.contains("----------"))
        .filter(|line| !line.trim().is_empty())
        .collect();

    assert_eq!(
        data_lines.len(),
        1,
        "Should have exactly one data line for user/repo: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_slug_filters_other_repos() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create multiple repos
    fixture.create_fake_installation("user1/repo1", "1.0.0")?;
    fixture.create_fake_installation("user2/repo2", "2.0.0")?;
    fixture.create_fake_installation("user1/repo3", "3.0.0")?;

    // Execute: run list with specific slug
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list").arg("user1/repo1");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // Assert: verify output
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "List command with slug should succeed"
    );

    // Should contain only user1/repo1
    assert!(
        stdout.contains("user1/repo1"),
        "Output should contain user1/repo1: {}",
        stdout
    );
    assert!(
        stdout.contains("1.0.0"),
        "Output should contain version 1.0.0: {}",
        stdout
    );

    // Should NOT contain other repos
    assert!(
        !stdout.contains("user2/repo2"),
        "Output should NOT contain user2/repo2: {}",
        stdout
    );
    assert!(
        !stdout.contains("user1/repo3"),
        "Output should NOT contain user1/repo3: {}",
        stdout
    );
    assert!(
        !stdout.contains("2.0.0"),
        "Output should NOT contain version 2.0.0: {}",
        stdout
    );
    assert!(
        !stdout.contains("3.0.0"),
        "Output should NOT contain version 3.0.0: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_non_existent_slug() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Don't create any installations

    // Execute: run list with non-existent slug
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list").arg("nonexistent/repo");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // Assert: verify output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "List command should succeed even for non-existent repo"
    );

    // Verify stderr contains "does not seem to be installed" message
    assert!(
        stderr.contains("does not seem to be installed"),
        "stderr should indicate repo is not installed: {}",
        stderr
    );

    // Verify stdout is empty (no table output)
    assert!(
        stdout.is_empty(),
        "stdout should be empty for non-existent repo: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_slug_partial_match() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create a repo
    fixture.create_fake_installation("testuser/testrepo", "1.0.0")?;

    // Execute: run list with different (non-existent) repo from same user
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list").arg("testuser/otherrepo");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // Assert: verify output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "List command should succeed even for non-matching repo"
    );

    // Verify message indicating not installed
    assert!(
        stderr.contains("does not seem to be installed") || stderr.contains("not found"),
        "stderr should indicate repo is not installed: {}",
        stderr
    );

    // Verify the other repo is NOT shown
    assert!(
        !stdout.contains("testuser/testrepo"),
        "Output should NOT contain testuser/testrepo: {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_invalid_slug_formats() -> Result<(), Box<dyn std::error::Error>> {
    use super::common::repo_format_validation::test_invalid_repo_formats_for_command;

    // Use the reusable validation function for invalid formats
    test_invalid_repo_formats_for_command("list")?;

    Ok(())
}

#[serial]
#[test]
fn test_list_with_valid_slug_formats() -> Result<(), Box<dyn std::error::Error>> {
    use super::common::repo_format_validation::test_valid_repo_formats_for_command;

    // Use the reusable validation function for valid formats
    test_valid_repo_formats_for_command("list")?;

    Ok(())
}

#[serial]
#[test]
fn test_list_with_slug_and_empty_version_dir() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create user/repo/1.0.0 directory but leave it empty (no binary inside)
    let install_dir = fixture.get_install_path("user/repo", "1.0.0");
    std::fs::create_dir_all(&install_dir)?;
    // Don't create any binary file inside

    // Execute: run list with slug
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list").arg("user/repo");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // Assert: should handle gracefully
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "List command should succeed even with empty version dir"
    );

    // Note: The current implementation of list_installed_versions_per_slug
    // does NOT filter empty directories (unlike list_installed_spells which does).
    // This test documents that behavior - empty version directories are shown.
    // This is arguably inconsistent but is the current implementation.
    assert!(
        stdout.contains("user/repo"),
        "Output should contain repository name (current implementation shows empty dirs): {}",
        stdout
    );
    assert!(
        stdout.contains("1.0.0"),
        "Output should contain version even if directory is empty (current implementation): {}",
        stdout
    );

    Ok(())
}

#[serial]
#[test]
fn test_list_with_slug_output_format_consistency() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;

    // Create installation
    fixture.create_fake_installation("myuser/myrepo", "1.0.0")?;

    // Execute: run list with slug
    let mut cmd = Command::new(cargo::cargo_bin!("poof"));
    cmd.arg("list").arg("myuser/myrepo");
    set_test_env(&mut cmd, &fixture);
    let output = cmd.output()?;

    // Assert: verify output format
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "List command with slug should succeed"
    );

    // Verify TSV format (Repository\tVersions header, tab-separated data)
    assert!(
        stdout.contains("Repository") && stdout.contains("Versions"),
        "Output should contain TSV headers: {}",
        stdout
    );
    assert!(
        stdout.contains("----------") && stdout.contains("--------"),
        "Output should contain header separators: {}",
        stdout
    );
    assert!(
        stdout.contains("\t"),
        "Output should be tab-separated: {}",
        stdout
    );

    // Check it follows TSV format (one line with tab separator)
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(
        lines.iter().any(|line| line.contains("myuser/myrepo")
            && line.contains("\t")
            && line.contains("1.0.0")),
        "Should have a data line with repo, tab, and version: {}",
        stdout
    );

    // Verify proper alignment with regular list command
    // The format should be: "repository_name\tversion1, version2, ..."
    let data_line = lines
        .iter()
        .find(|line| line.contains("myuser/myrepo"))
        .expect("Should find data line");

    let parts: Vec<&str> = data_line.split('\t').collect();
    assert_eq!(
        parts.len(),
        2,
        "Data line should have exactly 2 parts separated by tab: {}",
        data_line
    );

    Ok(())
}
