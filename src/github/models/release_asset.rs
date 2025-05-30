use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
/// Represents a release asset from GitHub.
pub struct ReleaseAsset {
    name: String,
    //size: u64,
    browser_download_url: String,
}
impl ReleaseAsset {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn browser_download_url(&self) -> &String {
        &self.browser_download_url
    }
}
