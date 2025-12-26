//! Mock GitHub API server for testing without network calls

use mockito::{Matcher, Mock, ServerGuard};
use serde_json::json;

/// Helper to create a mock GitHub API server
pub struct MockGitHub {
    pub server: ServerGuard,
}

impl MockGitHub {
    /// Create a new mock GitHub API server
    pub fn new() -> Self {
        let server = mockito::Server::new();
        Self { server }
    }

    /// Get the base URL for the mock server (to be set as POOF_GITHUB_API_URL)
    pub fn base_url(&self) -> String {
        self.server.url()
    }

    /// Mock a release endpoint for a specific repo
    /// Returns a mock that will respond with the given release data
    pub fn mock_latest_release(&mut self, repo: &str, tag: &str, assets: Vec<MockAsset>) -> Mock {
        let assets_json: Vec<_> = assets
            .iter()
            .map(|asset| {
                json!({
                    "name": asset.name,
                    "browser_download_url": asset.download_url,
                    "content_type": asset.content_type,
                })
            })
            .collect();

        self.server
            .mock("GET", format!("/{}/releases/latest", repo).as_str())
            .match_header("User-Agent", "pirafrank/poof")
            .match_header("Accept", "application/vnd.github.v3+json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "tag_name": tag,
                    "published_at": "2024-01-01T00:00:00Z",
                    "assets": assets_json,
                })
                .to_string(),
            )
            .create()
    }

    /// Mock a specific release by tag
    #[allow(dead_code)]
    pub fn mock_release_by_tag(&mut self, repo: &str, tag: &str, assets: Vec<MockAsset>) -> Mock {
        let assets_json: Vec<_> = assets
            .iter()
            .map(|asset| {
                json!({
                    "name": asset.name,
                    "browser_download_url": asset.download_url,
                    "content_type": asset.content_type,
                })
            })
            .collect();

        self.server
            .mock("GET", format!("/{}/releases/tags/{}", repo, tag).as_str())
            .match_header("User-Agent", "pirafrank/poof")
            .match_header("Accept", "application/vnd.github.v3+json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "tag_name": tag,
                    "published_at": "2024-01-01T00:00:00Z",
                    "assets": assets_json,
                })
                .to_string(),
            )
            .create()
    }

    /// Mock a 404 response for a repo (not found)
    #[allow(dead_code)]
    pub fn mock_not_found(&mut self, repo: &str) -> Mock {
        self.server
            .mock("GET", Matcher::Regex(format!("/{}/releases/.*", repo)))
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "message": "Not Found",
                })
                .to_string(),
            )
            .create()
    }

    /// Mock a network error
    pub fn mock_network_error(&mut self, repo: &str) -> Mock {
        self.server
            .mock("GET", Matcher::Regex(format!("/{}/releases/.*", repo)))
            .with_status(500)
            .with_body("Internal Server Error")
            .create()
    }

    /// Mock poof self-update check returning the given version
    pub fn mock_poof_update_get_version(&mut self, version: &str) -> Mock {
        self.mock_latest_release(
            "pirafrank/poof",
            version,
            vec![MockAsset {
                name: "poof-linux-x86_64".to_string(),
                download_url: format!(
                    "{}/releases/download/{}/poof-linux-x86_64",
                    self.base_url(),
                    version
                ),
                content_type: "application/octet-stream".to_string(),
            }],
        )
    }
}

/// Represents a mock GitHub release asset
pub struct MockAsset {
    pub name: String,
    pub download_url: String,
    pub content_type: String,
}

impl MockAsset {
    #[allow(dead_code)]
    pub fn new(name: &str, download_url: &str) -> Self {
        Self {
            name: name.to_string(),
            download_url: download_url.to_string(),
            content_type: "application/octet-stream".to_string(),
        }
    }
}
