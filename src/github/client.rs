//! GitHub API interaction for fetching releases and assets.

use anyhow::{anyhow, bail, Result};
use log::{debug, error, info, warn};

use crate::core::selector::get_env_compatible_assets;

use super::models::{Release, ReleaseAsset};

const GITHUB_API_URL: &str = "https://api.github.com/repos";
const GITHUB_API_USER_AGENT: &str = "pirafrank/poof";
const GITHUB_API_ACCEPT: &str = "application/vnd.github.v3+json";

/// Get the base API URL from environment or use the default
fn get_base_api_url() -> String {
    std::env::var("POOF_GITHUB_API_URL").unwrap_or_else(|_| GITHUB_API_URL.to_string())
}

pub fn get_release(repo: &str, tag: Option<&str>) -> Result<Release> {
    let release_url = get_release_url(repo, tag);
    info!("Release URL: {}", release_url);
    let client = reqwest::blocking::Client::new();

    // Make the request
    match client
        .get(&release_url)
        .header("User-Agent", GITHUB_API_USER_AGENT) // Keep User-Agent header for GitHub API
        .header("Accept", GITHUB_API_ACCEPT)
        .send()
    {
        Ok(response) => {
            debug!("Response Status: {}", response.status());
            let status = response.status(); // we store for error case

            if response.status().is_success() {
                // Attempt to parse the JSON response into a Vec<Release>
                match response.json::<Release>() {
                    Ok(release) => {
                        if let Some(tag) = tag {
                            info!("Selected release tag: {}", tag);
                        } else {
                            info!("Current latest release tag: {}", release.tag_name());
                        }
                        debug!("Published at: {}", release.published_at());
                        debug!("Available assets:");
                        for asset in release.assets() {
                            debug!("\t{}", asset.name());
                        }
                        // return Ok on success
                        Ok(release)
                    }
                    Err(e) => {
                        error!("Failed to parse JSON response: {}", e);
                        // return Err instead of exit, wrapping the original error
                        Err(anyhow!(e).context(format!(
                            "Failed to parse JSON response from {}",
                            release_url
                        )))
                    }
                }
            } else {
                error!("Request failed with status: {}", status);
                // read body for context if possible
                let error_body = response
                    .text()
                    .unwrap_or_else(|_| "Failed to read error response body".to_string());
                // return Err instead of exit
                Err(anyhow!(
                    "Request to {} failed with status: {}. Response: {}",
                    release_url,
                    status,
                    error_body
                ))
            }
        }
        Err(e) => {
            error!("Failed to send request: {}", e);
            // return Err instaed of exit
            Err(anyhow!(e).context(format!("Failed to send request to {}", release_url)))
        }
    }
}

pub fn get_release_url(repo: &str, tag: Option<&str>) -> String {
    let base_url = get_base_api_url();
    match tag {
        Some(tag) => format!("{}/{}/releases/tags/{}", base_url, repo, tag),
        None => format!("{}/{}/releases/latest", base_url, repo),
    }
}

pub fn get_asset(release: &Release) -> Result<ReleaseAsset> {
    let binaries: Option<Vec<ReleaseAsset>> =
        get_env_compatible_assets(release.assets(), |asset| asset.name());
    let not_found = format!(
        "No compatible pre-built binaries found for release {} matching the specified criteria.",
        release.tag_name()
    );

    if binaries.is_none() {
        bail!(not_found);
    }

    let binaries = binaries.unwrap();
    if binaries.is_empty() {
        bail!(not_found);
    }

    debug!("Compatible binaries found:");
    for binary in &binaries {
        debug!("\t{}", binary.name());
    }
    if binaries.len() > 1 {
        warn!(
            "Multiple compatible binaries found for release {}. Selecting first: {}",
            release.tag_name(),
            binaries[0].name()
        );
        // TODO: allow to specify which binary to download via explicit URL given to 'install' command
    }
    // Return the first compatible binary
    Ok(binaries[0].clone())
}

#[cfg(test)]
mod tests;
