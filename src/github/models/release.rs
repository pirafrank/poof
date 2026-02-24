use serde::{Deserialize, Serialize};

use super::ReleaseAsset;

/// A GitHub release as returned by the GitHub REST API.
#[derive(Deserialize, Serialize, Debug)]
pub struct Release {
    /// The version tag of the release (e.g. `"v1.2.3"`).
    tag_name: String,
    /// ISO 8601 timestamp of when the release was published.
    published_at: String,
    /// List of release assets attached to this release.
    assets: Vec<ReleaseAsset>,
}

impl Release {
    /// Returns the release tag name.
    pub fn tag_name(&self) -> &String {
        &self.tag_name
    }

    /// Returns the publication timestamp string.
    pub fn published_at(&self) -> &String {
        &self.published_at
    }

    /// Returns the list of assets attached to this release.
    pub fn assets(&self) -> &Vec<ReleaseAsset> {
        &self.assets
    }
}
