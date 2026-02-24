//! Main file handling 'unlink' command

use anyhow::{bail, Context, Result};
use log::info;
use std::io::{self, Write};

use crate::cli::UnlinkArgs;
use crate::files::datadirs;

/// Remove a binary's symlink from the bin directory, making it unavailable in `PATH`.
///
/// Only symlinks managed by poof are removed. Regular files are refused to prevent
/// accidental deletion of foreign binaries. The user is prompted for confirmation
/// unless the `--yes` / `-y` flag is set.
pub fn run_unlink(args: &UnlinkArgs) -> Result<()> {
    let bin_dir = datadirs::get_bin_dir().context("Cannot get bin directory path")?;
    let binary_path = bin_dir.join(&args.binary_name);

    // Check if binary exists
    if !binary_path.exists() {
        bail!(
            "Binary '{}' not found in bin directory. \
Check installed binaries using 'list' command.",
            args.binary_name
        );
    }

    // Verify it's a symlink
    if !binary_path.is_symlink() {
        bail!(
            "Binary '{}' exists but is not a symlink. \
            Refusing to delete regular files.\n\
            This is likely to be a foreign binary not managed by poof.\n\
            Please remove it manually and try again.",
            args.binary_name
        );
    }

    // Skip confirmation if -y flag is set
    if !args.yes {
        // Show what will be deleted
        println!("This will remove '{}' from PATH.", args.binary_name);

        // Ask for confirmation
        print!("Proceed? (y/yes): ");
        io::stdout().flush().context("Cannot flush stdout")?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Cannot read user input")?;

        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            info!("Unlinking cancelled.");
            return Ok(());
        }
    }

    // Delete the symlink
    info!("Removing '{}' from PATH...", args.binary_name);
    std::fs::remove_file(&binary_path).with_context(|| {
        format!(
            "Cannot remove '{}' from PATH: {}",
            args.binary_name,
            binary_path.display()
        )
    })?;

    info!(
        "'{}' successfully removed from PATH. Use 'poof use' to re-add it.",
        args.binary_name
    );
    Ok(())
}
