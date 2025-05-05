//! Main file handling 'download' command

use anyhow::{Context, Result};
use log::{debug, info};
use std::{fs::File, io::copy, path::PathBuf};

// Function to handle downloading and potentially installing binaries
pub fn download_binary(
    filename: &String,
    download_url: &String,
    download_to: &PathBuf,
) -> Result<()> {
    info!("Downloading {}\nfrom {}", filename, download_url);

    let response = reqwest::blocking::get(download_url)
        .with_context(|| format!("Failed to initiate download from {}", download_url))?;

    let status = response.status(); // for borrowing
    if status.is_success() {
        // Ensure the directory exists
        std::fs::create_dir_all(download_to)
            .with_context(|| format!("Failed to create directory {}", download_to.display()))?;

        // Create the file path and open it for writing
        let archive_path = download_to.join(filename);
        let mut file = File::create(&archive_path)
            .with_context(|| format!("Failed to create file {}", archive_path.display()))?;

        debug!("Saving to: {}", archive_path.display());

        // Copy the response body to the file
        let content = response
            .bytes()
            .context("Failed to read download response bytes")?; // Use context
        copy(&mut content.as_ref(), &mut file)
            .context("Failed to write downloaded data to file")?;

        info!("Download complete.");
        Ok(())
    } else {
        // we use anyhow::bail! for errors originating here
        let error_body = response
            .text()
            .unwrap_or_else(|_| "Failed to read error body".to_string());
        anyhow::bail!(
            // with bail! macro we early return with error
            "Download failed! Status: {}. URL: {}. Server response: {}",
            status,
            download_url,
            error_body
        );
        // the error will be propagated and logged at a higher level in main.rs
        // also, we have context added via `?` to trace the origin :)
    }
}
