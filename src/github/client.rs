//! GitHub API interaction for fetching releases and assets.

use log::{debug, error, info, warn};

use crate::github::models::Release;

use super::models::ReleaseAsset;

const GITHUB_API_URL: &str = "https://api.github.com/repos";
const GITHUB_API_USER_AGENT: &str = "pirafrank/poof";
const GITHUB_API_ACCEPT: &str = "application/vnd.github.v3+json";

pub fn get_release(repo: &str, tag: Option<&str>) -> Release {
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
            if response.status().is_success() {
                // Attempt to parse the JSON response into a Vec<Release>
                match response.json::<Release>() {
                    Ok(release) => {
                        if tag.is_some() {
                            info!("Selected release tag: {}", tag.unwrap());
                        } else {
                            info!("Current latest release tag: {}", release.tag_name());
                        }
                        info!("Published at: {}", release.published_at());
                        debug!("Available assets:");
                        for asset in release.assets() {
                            debug!("\t{}", asset.name());
                        }
                        release
                    }
                    Err(e) => {
                        error!("Failed to parse JSON response: {}", e);
                        std::process::exit(101);
                    }
                }
            } else {
                error!("Request failed with status: {}", response.status());
                std::process::exit(102);
            }
        }
        Err(e) => {
            error!("Failed to send request: {}", e);
            std::process::exit(91);
        }
    }
}

pub fn get_release_url(repo: &str, tag: Option<&str>) -> String {
    match tag {
        Some(tag) => format!("{}/{}/releases/tags/{}", GITHUB_API_URL, repo, tag),
        None => format!("{}/{}/releases/latest", GITHUB_API_URL, repo),
    }
}

pub fn get_asset<F>(release: &Release, f: F) -> ReleaseAsset
where
    F: Fn(&str) -> bool,
{
    let binaries: Vec<ReleaseAsset> = release
        .assets()
        .iter()
        .filter(|asset| f(asset.name()))
        .cloned()
        .collect();

    if binaries.is_empty() {
        error!("No compatible pre-built binaries found.");
        std::process::exit(100);
    }
    debug!("Compatible binaries found:");
    for binary in &binaries {
        debug!("\t{}", binary.name());
    }
    if binaries.len() > 1 {
        warn!("Multiple compatible binaries found. Downloading first...");
        // TODO: allow to specify which binary to download via explicit URL given to 'install' command
    }
    // Return the first compatible binary
    binaries[0].clone()
}
