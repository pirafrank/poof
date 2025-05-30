//! Main file handling 'cleanup' command

use anyhow::{Context, Result};
use log::info;
use std::io::{self, Write};

use crate::files::datadirs;

pub fn run_clean() -> Result<()> {
    let cache_dir = datadirs::get_cache_dir().context("Failed to get cache directory path")?;

    // fallback albeit cache and data dirs are created at startup
    // yet if this behaviour changes, we should handle it here.
    if !cache_dir.exists() {
        info!("Nothing to clean up. Cache directory does not exist.");
        return Ok(());
    }

    // Show what will be deleted
    info!(
        "This will delete the cache directory: {}",
        cache_dir.display()
    );

    // Ask for confirmation
    print!("Proceed? (y/yes): ");
    io::stdout().flush().context("Failed to flush stdout")?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .context("Failed to read user input")?;

    let input = input.trim().to_lowercase();

    if input == "y" || input == "yes" {
        info!("Deleting cache directory...");

        std::fs::remove_dir_all(&cache_dir).with_context(|| {
            format!("Failed to delete cache directory: {}", cache_dir.display())
        })?;

        info!("Cache directory successfully deleted.");
    } else {
        info!("Cleanup cancelled.");
    }

    Ok(())
}
