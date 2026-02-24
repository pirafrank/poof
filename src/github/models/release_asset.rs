use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, Debug)]
/// Represents a release asset from GitHub.
pub struct ReleaseAsset {
    /// File name of the asset (e.g. `"mytool-linux-x86_64.tar.gz"`).
    name: String,
    /// Direct download URL for the asset.
    browser_download_url: String,
}
impl ReleaseAsset {
    /// Returns the asset file name.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns the browser-accessible download URL for the asset.
    pub fn browser_download_url(&self) -> &String {
        &self.browser_download_url
    }
}
