use serde::Deserialize;

use super::ReleaseAsset;

#[derive(Deserialize, Debug)]
pub struct Release {
    tag_name: String,
    published_at: String, // Consider using chrono::DateTime<chrono::Utc> for proper date handling
    assets: Vec<ReleaseAsset>,
}

impl Release {
    pub fn new(tag_name: String, published_at: String, assets: Vec<ReleaseAsset>) -> Self {
        Self {
            tag_name,
            published_at,
            assets,
        }
    }

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
