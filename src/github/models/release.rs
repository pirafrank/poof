use serde::{Deserialize, Serialize};

use super::ReleaseAsset;

#[derive(Deserialize, Serialize, Debug)]
/// Represents a GitHub release.
/// The `tag_name` is the version tag of the release.
/// The `published_at` is the date when the release was published.
/// The `assets` is a list of assets associated with the release.
/// The `Release` struct is used to deserialize the JSON response from the GitHub API.
pub struct Release {
    tag_name: String,
    published_at: String, // Consider using chrono::DateTime<chrono::Utc> for proper date handling
    assets: Vec<ReleaseAsset>,
}

impl Release {
    pub fn tag_name(&self) -> &String {
        &self.tag_name
    }

    pub fn published_at(&self) -> &String {
        &self.published_at
    }

    pub fn assets(&self) -> &Vec<ReleaseAsset> {
        &self.assets
    }
}
