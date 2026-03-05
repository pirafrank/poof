//! Main file handling 'download' command

use anyhow::{Context, Result};
use log::{debug, info};
use reqwest::blocking::Client;
use std::{fs::File, io::copy, path::PathBuf};

/// Accept header value for downloading release assets via the GitHub API.
const GITHUB_ASSET_ACCEPT: &str = "application/octet-stream";
/// `User-Agent` header value sent with every GitHub API request.
const GITHUB_API_USER_AGENT: &str = "pirafrank/poof";

/// Download a single release asset to a local directory.
///
/// When `token` is `Some`, the download uses the GitHub API URL with
/// `Authorization: Bearer` and `Accept: application/octet-stream` headers,
/// which is required for assets from private repositories. When `token` is
/// `None` the asset is fetched without auth via a plain GET request.
///
/// Fetches `download_url` and writes the response body to `download_to/filename`.
/// The destination directory is created if it does not already exist.
/// Returns the full path of the saved file on success.
pub fn download_asset(
    filename: &String,
    download_url: &String,
    download_to: &PathBuf,
    token: Option<&str>,
) -> Result<PathBuf> {
    info!("Downloading {} from {}", filename, download_url);

    let client = Client::new();
    let mut request = client
        .get(download_url)
        .header("User-Agent", GITHUB_API_USER_AGENT);

    if let Some(token) = token {
        debug!("Using authenticated download for {}", filename);
        request = request
            .header("Authorization", format!("Bearer {}", token))
            .header("Accept", GITHUB_ASSET_ACCEPT);
    }

    let response = request
        .send()
        .with_context(|| format!("Cannot initiate download from {}", download_url))?;

    let status = response.status(); // for borrowing
    if status.is_success() {
        // Ensure the directory exists
        std::fs::create_dir_all(download_to)
            .with_context(|| format!("Cannot create directory {}", download_to.display()))?;

        // Create the file path and open it for writing
        let target_file_path = download_to.join(filename);
        let mut file = File::create(&target_file_path)
            .with_context(|| format!("Cannot create file {}", target_file_path.display()))?;

        debug!("Saving to: {}", target_file_path.display());

        // Copy the response body to the file
        let content = response
            .bytes()
            .context("Cannot read download response bytes")?; // Use context
        copy(&mut content.as_ref(), &mut file).context("Cannot write downloaded data to file")?;

        info!("Download complete.\n");
        Ok(target_file_path.clone())
    } else {
        // we use anyhow::bail! for errors originating here
        let error_body = response
            .text()
            .unwrap_or_else(|_| "Cannot read error body".to_string());
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

#[cfg(test)]
mod tests;
