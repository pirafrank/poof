//! GitHub API interaction for fetching releases and assets.

use anyhow::{anyhow, bail, Context, Result};
use log::{debug, error, info};
use reqwest::blocking::{Client, RequestBuilder};

use crate::core::selector::get_env_compatible_assets;

use super::models::{Release, ReleaseAsset};

const GITHUB_API_URL: &str = "https://api.github.com/repos";
const GITHUB_API_USER_AGENT: &str = "pirafrank/poof";
const GITHUB_API_ACCEPT: &str = "application/vnd.github.v3+json";

fn get_github_token() -> Result<String> {
    let token = std::env::var("GITHUB_TOKEN").with_context(|| "GITHUB_TOKEN is not set")?;
    if token.is_empty() {
        bail!("GITHUB_TOKEN is not set");
    }
    Ok(token)
}

/// Get the base API URL from environment or use the default
fn get_base_api_url() -> String {
    std::env::var("POOF_GITHUB_API_URL").unwrap_or_else(|_| GITHUB_API_URL.to_string())
}

/// Fetch a GitHub release for `repo`.
///
/// When `tag` is `None` the latest release is retrieved. When a tag string is
/// provided that specific release tag is fetched. Attaches a `Bearer` token
/// from the `GITHUB_TOKEN` environment variable when available to avoid rate
/// limiting. The base API URL can be overridden via `POOF_GITHUB_API_URL`
/// (useful in tests with a mock server).
pub fn get_release(repo: &str, tag: Option<&str>) -> Result<Release> {
    let release_url = get_release_url(repo, tag);
    info!("Release URL: {}", release_url);
    let client: Client = Client::new();

    let mut request: RequestBuilder = client
        .get(&release_url)
        .header("User-Agent", GITHUB_API_USER_AGENT) // Keep User-Agent header for GitHub API
        .header("Accept", GITHUB_API_ACCEPT);

    // Add Authorization header if token is available to avoid rate limiting
    if let Ok(token) = get_github_token() {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    // Make the request
    match request.send() {
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
                        error!("Cannot parse JSON response: {}", e);
                        // return Err instead of exit, wrapping the original error
                        Err(anyhow!(e)
                            .context(format!("Cannot parse JSON response from {}", release_url)))
                    }
                }
            } else {
                error!("Request failed with status: {}", status);
                // read body for context if possible
                let error_body = response
                    .text()
                    .unwrap_or_else(|_| "Cannot read error response body".to_string());
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
            error!("Failed: {}. Are you connected to the internet?", e);
            // return Err instaed of exit
            Err(anyhow!(e).context(format!("Cannot send request to {}", release_url)))
        }
    }
}

/// Build the GitHub API URL for a release.
///
/// Returns the `/releases/tags/{tag}` endpoint when a specific tag is requested
/// or the `/releases/latest` endpoint otherwise.
pub fn get_release_url(repo: &str, tag: Option<&str>) -> String {
    let base_url = get_base_api_url();
    match tag {
        Some(tag) => format!("{}/{}/releases/tags/{}", base_url, repo, tag),
        None => format!("{}/{}/releases/latest", base_url, repo),
    }
}

/// Filter a release's assets to those compatible with the current platform.
///
/// Delegates to [`get_env_compatible_assets`] and returns an error when no
/// compatible assets are found for the release.
pub fn get_assets(release: &Release) -> Result<Vec<ReleaseAsset>> {
    let binaries: Option<Vec<ReleaseAsset>> =
        get_env_compatible_assets(release.assets(), |asset| asset.name());
    let not_found = format!(
        "No compatible pre-built binaries found for release {} matching the specified criteria.",
        release.tag_name()
    );

    if binaries.is_none() {
        bail!(not_found);
    }

    let binaries: Vec<ReleaseAsset> = binaries.unwrap_or_default();
    if binaries.is_empty() {
        bail!(not_found);
    }

    debug!("Compatible binaries found:");
    for binary in &binaries {
        debug!("\t{}", binary.name());
    }
    Ok(binaries)
}

#[cfg(test)]
mod tests;
