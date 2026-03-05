use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
/// Represents a release asset from GitHub.
pub struct ReleaseAsset {
    /// File name of the asset (e.g. `"mytool-linux-x86_64.tar.gz"`).
    name: String,
    /// GitHub API URL for the asset (used for authenticated downloads of private repos).
    url: String,
    /// Direct download URL for the asset (only works for public repos without auth).
    browser_download_url: String,
}
impl ReleaseAsset {
    /// Returns the asset file name.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns the GitHub API URL for the asset.
    /// Use this with `Authorization` + `Accept: application/octet-stream` headers
    /// when downloading from private repositories.
    pub fn url(&self) -> &String {
        &self.url
    }

    /// Returns the browser-accessible download URL for the asset.
    /// Only works for public repositories without authentication.
    pub fn browser_download_url(&self) -> &String {
        &self.browser_download_url
    }
}
