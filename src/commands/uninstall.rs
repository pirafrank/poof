//! Main file handling 'uninstall' command

use anyhow::{bail, Context, Result};
use log::{debug, info};
use std::fs;
use std::io::{stdin, stdout, Write};
use std::path::Path;

use crate::cli::UninstallArgs;
use crate::files::datadirs;
use crate::files::filesys::is_broken_symlink;

pub fn run_uninstall(args: &UninstallArgs) -> Result<()> {
    let data_dir = datadirs::get_data_dir().context("Cannot get data directory")?;
    let bin_dir = datadirs::get_bin_dir().context("Cannot get bin directory")?;

    // Determine what to delete and set a proper message for the confirmation prompt.
    let (target_path, message) = if args.all {
        let path = datadirs::get_versions_nest(&data_dir, &args.repo);
        let msg = format!(
            "This will delete ALL versions of '{}' and remove provided binaries from PATH.",
            args.repo
        );
        (path, msg)
    } else if let Some(version) = &args.version {
        let path = datadirs::get_binary_nest(&data_dir, &args.repo, version);
        let msg = format!(
            "This will delete version '{}' of '{}' and remove provided binaries from PATH.",
            version, args.repo
        );
        (path, msg)
    } else {
        // This shouldn't happen due to clap validation, but handle it gracefully
        bail!("Please specify either --version or --all flags.");
    };

    // Check if the target exists
    if !target_path.exists() {
        if args.all {
            info!("No versions of '{}' installed. Nothing to do.", args.repo);
        } else if let Some(version) = &args.version {
            info!(
                "Version '{}' of '{}' is not installed. Nothing to do.",
                version, args.repo
            );
        }
        return Ok(());
    }

    // Show what will be deleted
    info!("{}", message);
    debug!(
        "Uninstalling '{}' by removing directory: {}",
        args.repo,
        target_path.display()
    );

    // Skip confirmation if -y flag is set
    if !args.yes {
        // Ask for confirmation
        print!("Proceed? (y/yes): ");
        stdout().flush().context("Cannot flush stdout")?;

        let mut input = String::new();
        stdin()
            .read_line(&mut input)
            .context("Cannot read user input")?;

        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            info!("Uninstall cancelled.");
            return Ok(());
        }
    }

    // Delete the directory
    debug!("Deleting directory: {}", target_path.display());
    fs::remove_dir_all(&target_path)
        .with_context(|| format!("Cannot delete directory: {}", target_path.display()))?;

    if args.all {
        info!(
            "All versions of '{}' have been successfully removed.",
            args.repo
        );
    } else if let Some(version) = &args.version {
        info!(
            "Version '{}' of '{}' has been successfully removed.",
            version, args.repo
        );
    }

    // Clean up broken symlinks
    let cleaned_count =
        clean_broken_symlinks(&bin_dir).context("Failed to clean broken symlinks")?;

    if cleaned_count > 0 {
        debug!(
            "Removed {} broken symlink(s) from bin directory.",
            cleaned_count
        );
    }

    Ok(())
}

/// Clean broken symlinks from the bin directory.
/// Returns the number of symlinks that were removed.
fn clean_broken_symlinks(bin_dir: &Path) -> Result<usize> {
    let mut count = 0;

    // Return early if bin_dir doesn't exist
    if !bin_dir.exists() {
        return Ok(0);
    }

    let entries = fs::read_dir(bin_dir)
        .with_context(|| format!("Cannot read bin directory: {}", bin_dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();

        // Check if it's a symlink
        if is_broken_symlink(&path)? {
            fs::remove_file(&path)
                .with_context(|| format!("Cannot remove broken symlink: {}", path.display()))?;
            count += 1;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper to create a test environment
    struct TestEnv {
        _temp_dir: TempDir,
        bin_dir: std::path::PathBuf,
    }

    impl TestEnv {
        fn new() -> Result<Self> {
            let temp_dir = TempDir::new()?;
            let bin_dir = temp_dir.path().join("bin");
            fs::create_dir_all(&bin_dir)?;

            Ok(Self {
                _temp_dir: temp_dir,
                bin_dir,
            })
        }

        #[cfg(not(target_os = "windows"))]
        fn create_symlink(&self, name: &str, target: &Path) -> Result<std::path::PathBuf> {
            let link_path = self.bin_dir.join(name);
            std::os::unix::fs::symlink(target, &link_path)?;
            Ok(link_path)
        }
    }

    #[test]
    fn test_clean_broken_symlinks_empty_dir() -> Result<()> {
        let env = TestEnv::new()?;
        let count = clean_broken_symlinks(&env.bin_dir)?;
        assert_eq!(count, 0, "Empty directory should return 0");
        Ok(())
    }

    #[test]
    fn test_clean_broken_symlinks_nonexistent_dir() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let nonexistent = temp_dir.path().join("nonexistent");
        let count = clean_broken_symlinks(&nonexistent)?;
        assert_eq!(count, 0, "Nonexistent directory should return 0");
        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_clean_broken_symlinks_no_broken() -> Result<()> {
        let env = TestEnv::new()?;

        // Create a valid target file
        let target = env.bin_dir.join("target_file");
        fs::write(&target, b"content")?;

        // Create a valid symlink pointing to the target
        env.create_symlink("valid_link", &target)?;

        let count = clean_broken_symlinks(&env.bin_dir)?;
        assert_eq!(count, 0, "No broken symlinks should return 0");

        // Verify the symlink still exists
        assert!(env.bin_dir.join("valid_link").exists());

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_clean_broken_symlinks_removes_broken_only() -> Result<()> {
        let env = TestEnv::new()?;

        // Create a valid target and symlink
        let valid_target = env.bin_dir.join("valid_target");
        fs::write(&valid_target, b"content")?;
        env.create_symlink("valid_link", &valid_target)?;

        // Create a broken symlink (target doesn't exist)
        let broken_target = env.bin_dir.join("nonexistent_target");
        env.create_symlink("broken_link", &broken_target)?;

        let count = clean_broken_symlinks(&env.bin_dir)?;
        assert_eq!(count, 1, "Should remove exactly 1 broken symlink");

        // Verify valid symlink still exists
        assert!(env.bin_dir.join("valid_link").exists());

        // Verify broken symlink was removed
        assert!(!env.bin_dir.join("broken_link").exists());

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_clean_broken_symlinks_mixed() -> Result<()> {
        let env = TestEnv::new()?;

        // Create regular file
        fs::write(env.bin_dir.join("regular_file"), b"content")?;

        // Create valid symlink
        let valid_target = env.bin_dir.join("valid_target");
        fs::write(&valid_target, b"content")?;
        env.create_symlink("valid_link", &valid_target)?;

        // Create two broken symlinks
        env.create_symlink("broken_link1", &env.bin_dir.join("nonexistent1"))?;
        env.create_symlink("broken_link2", &env.bin_dir.join("nonexistent2"))?;

        let count = clean_broken_symlinks(&env.bin_dir)?;
        assert_eq!(count, 2, "Should remove exactly 2 broken symlinks");

        // Verify regular file and valid symlink still exist
        assert!(env.bin_dir.join("regular_file").exists());
        assert!(env.bin_dir.join("valid_link").exists());

        // Verify broken symlinks were removed
        assert!(!env.bin_dir.join("broken_link1").exists());
        assert!(!env.bin_dir.join("broken_link2").exists());

        Ok(())
    }

    #[test]
    #[cfg(not(target_os = "windows"))]
    fn test_clean_broken_symlinks_multiple_broken() -> Result<()> {
        let env = TestEnv::new()?;

        // Create multiple broken symlinks
        for i in 0..5 {
            env.create_symlink(
                &format!("broken_{}", i),
                &env.bin_dir.join(format!("nonexistent_{}", i)),
            )?;
        }

        let count = clean_broken_symlinks(&env.bin_dir)?;
        assert_eq!(count, 5, "Should remove all 5 broken symlinks");

        // Verify all were removed
        for i in 0..5 {
            assert!(!env.bin_dir.join(format!("broken_{}", i)).exists());
        }

        Ok(())
    }
}
