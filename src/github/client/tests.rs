//! Unit tests for GitHub client functions
//! Tests without making actual network calls

use super::*;
use std::fs;

/// Helper function to load the fixture data
fn load_release_fixture() -> Release {
    let fixture_path =
        "tests/fixtures/responses/api.github.com/repos/pirafrank/poof/releases/latest";
    let json_data = fs::read_to_string(fixture_path).expect("Cannot read fixture file");
    serde_json::from_str(&json_data).expect("Cannot parse JSON fixture")
}

mod get_release_url {
    use super::*;

    #[test]
    fn test_latest_release_url() {
        let url = get_release_url("owner/repo", None);
        assert_eq!(
            url,
            "https://api.github.com/repos/owner/repo/releases/latest"
        );
    }

    #[test]
    fn test_specific_tag_release_url() {
        let url = get_release_url("owner/repo", Some("v1.0.0"));
        assert_eq!(
            url,
            "https://api.github.com/repos/owner/repo/releases/tags/v1.0.0"
        );
    }

    #[test]
    fn test_url_with_special_characters_in_repo() {
        let url = get_release_url("user-name/repo_name", None);
        assert_eq!(
            url,
            "https://api.github.com/repos/user-name/repo_name/releases/latest"
        );
    }

    #[test]
    fn test_url_with_special_characters_in_tag() {
        let url = get_release_url("owner/repo", Some("v1.0.0-beta.1"));
        assert_eq!(
            url,
            "https://api.github.com/repos/owner/repo/releases/tags/v1.0.0-beta.1"
        );
    }

    #[test]
    fn test_url_with_numeric_repo() {
        let url = get_release_url("owner123/repo456", Some("1.2.3"));
        assert_eq!(
            url,
            "https://api.github.com/repos/owner123/repo456/releases/tags/1.2.3"
        );
    }
}

mod release_model {
    use super::*;

    #[test]
    fn test_release_deserialization() {
        let release = load_release_fixture();

        // Verify the basic fields are correctly deserialized
        assert_eq!(release.tag_name(), "v0.5.0");
        assert_eq!(release.published_at(), "2025-06-16T20:32:32Z");
    }

    #[test]
    fn test_release_assets_have_names() {
        let release = load_release_fixture();

        // Verify all assets have non-empty names
        for asset in release.assets() {
            assert!(!asset.name().is_empty());
        }
    }

    #[test]
    fn test_release_assets_have_download_urls() {
        let release = load_release_fixture();

        // Verify all assets have browser download URLs
        for asset in release.assets() {
            assert!(!asset.browser_download_url().is_empty());
            assert!(asset.browser_download_url().starts_with("https://"));
        }
    }

    #[test]
    fn test_release_asset_download_url_format() {
        let release = load_release_fixture();

        // Find a specific asset and verify its download URL format
        let asset = release
            .assets()
            .iter()
            .find(|a| a.name() == "poof-0.5.0-x86_64-unknown-linux-gnu.tar.gz")
            .expect("Asset not found");

        assert_eq!(
            asset.browser_download_url(),
            "https://github.com/pirafrank/poof/releases/download/v0.5.0/poof-0.5.0-x86_64-unknown-linux-gnu.tar.gz"
        );
    }

    #[test]
    fn test_release_tag_name_format() {
        let release = load_release_fixture();

        // Verify tag name starts with 'v'
        assert!(release.tag_name().starts_with('v'));

        // Verify it contains version numbers
        assert!(release.tag_name().contains('.'));
    }

    #[test]
    fn test_release_published_at_format() {
        let release = load_release_fixture();

        // Verify published_at is in ISO 8601 format
        let published_at = release.published_at();
        assert!(published_at.contains('T'));
        assert!(published_at.contains('Z'));
        assert!(published_at.contains('-'));
        assert!(published_at.contains(':'));
    }

    #[test]
    fn test_all_platforms_represented() {
        let release = load_release_fixture();

        // Check that we have assets for all expected platforms
        let platforms = vec![
            "x86_64-unknown-linux-gnu",
            "x86_64-unknown-linux-musl",
            "aarch64-unknown-linux-gnu",
            "aarch64-unknown-linux-musl",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
        ];

        for platform in platforms {
            let found = release.assets().iter().any(|a| a.name().contains(platform));
            assert!(found, "Platform '{}' not found in assets", platform);
        }
    }

    #[test]
    fn test_sha256_files_exist_for_archives() {
        let release = load_release_fixture();

        // Get all .tar.gz files (excluding .sha256)
        let tar_gz_assets: Vec<_> = release
            .assets()
            .iter()
            .filter(|a| a.name().ends_with(".tar.gz"))
            .collect();

        // Verify each has a corresponding .sha256 file
        for asset in tar_gz_assets {
            let sha256_name = format!("{}.sha256", asset.name());
            let has_sha256 = release.assets().iter().any(|a| a.name() == &sha256_name);
            assert!(has_sha256, "Missing .sha256 file for {}", asset.name());
        }
    }
}

mod integration_with_fixture {
    use super::*;

    #[test]
    fn test_fixture_file_exists() {
        let fixture_path =
            "tests/fixtures/responses/api.github.com/repos/pirafrank/poof/releases/latest";
        assert!(
            std::path::Path::new(fixture_path).exists(),
            "Fixture file should exist"
        );
    }

    #[test]
    fn test_fixture_is_valid_json() {
        let fixture_path =
            "tests/fixtures/responses/api.github.com/repos/pirafrank/poof/releases/latest";
        let json_data = fs::read_to_string(fixture_path).expect("Cannot read fixture file");
        let parsed: serde_json::Value =
            serde_json::from_str(&json_data).expect("Fixture should be valid JSON");

        // Verify it has expected top-level fields
        assert!(parsed.get("tag_name").is_some());
        assert!(parsed.get("published_at").is_some());
        assert!(parsed.get("assets").is_some());
    }

    #[test]
    fn test_release_can_be_serialized_back_to_json() {
        let release = load_release_fixture();

        // Try to serialize it back (this tests that the struct is complete)
        let json = serde_json::to_string(&release);
        assert!(json.is_ok(), "Release should be serializable to JSON");
    }
}
