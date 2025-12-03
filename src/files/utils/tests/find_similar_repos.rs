use crate::files::utils::find_similar_repos;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper function to create a temporary directory structure for testing
fn setup_test_dir_structure() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    // Create user directories with repo subdirectories
    fs::create_dir_all(data_dir.join("user1/repo1")).unwrap();
    fs::create_dir_all(data_dir.join("user1/repo2")).unwrap();
    fs::create_dir_all(data_dir.join("user2/test-repo")).unwrap();
    fs::create_dir_all(data_dir.join("user2/another-repo")).unwrap();
    fs::create_dir_all(data_dir.join("pirafrank/rust_exif_renamer")).unwrap();
    fs::create_dir_all(data_dir.join("pirafrank/rust_exit_renamere")).unwrap();
    fs::create_dir_all(data_dir.join("octocat/Hello-World")).unwrap();

    temp_dir
}

#[test]
fn test_exact_match() {
    let temp_dir = setup_test_dir_structure();
    let data_dir = temp_dir.path();

    let results = find_similar_repos(data_dir, "user1/repo1");

    assert!(results.contains(&"user1/repo1".to_string()));
    assert_eq!(results[0], "user1/repo1");
}

#[test]
fn test_close_typo_match() {
    let temp_dir = setup_test_dir_structure();
    let data_dir = temp_dir.path();

    // "repo2" vs "repo1" - one character difference
    let results = find_similar_repos(data_dir, "user1/repo2");

    assert!(results.contains(&"user1/repo2".to_string()));
    assert!(results.contains(&"user1/repo1".to_string()));
}

#[test]
fn test_similar_repos_sorted_by_distance() {
    let temp_dir = setup_test_dir_structure();
    let data_dir = temp_dir.path();

    // "pirafrank/rust_exif_renamer" vs "pirafrank/rust_exit_renamere" - 2 char difference
    let results = find_similar_repos(data_dir, "pirafrank/rust_exif_renamer");

    assert!(!results.is_empty());
    // The exact match should be first
    assert_eq!(results[0], "pirafrank/rust_exif_renamer");
    // The similar one should be second
    assert_eq!(results[1], "pirafrank/rust_exit_renamere");
}

#[test]
fn test_distance_threshold_3() {
    let temp_dir = setup_test_dir_structure();
    let data_dir = temp_dir.path();

    // Test with a repo name that has distance <= 3 from "user1/repo1"
    // "user1/repa1" has distance 1 from "user1/repo1"
    fs::create_dir_all(data_dir.join("user1/repa1")).unwrap();

    let results = find_similar_repos(data_dir, "user1/repo1");

    assert!(results.contains(&"user1/repo1".to_string()));
    assert!(results.contains(&"user1/repa1".to_string()));
}

#[test]
fn test_percentage_similarity_threshold() {
    let temp_dir = setup_test_dir_structure();
    let data_dir = temp_dir.path();

    // Create repos with 70% similarity (30% distance)
    // "user1/testing" (13 chars) vs "user1/test-repo" (15 chars)
    fs::create_dir_all(data_dir.join("user1/testing")).unwrap();

    let results = find_similar_repos(data_dir, "user2/test-repo");

    // Should find test-repo
    assert!(results.contains(&"user2/test-repo".to_string()));
}

#[test]
fn test_no_similar_repos() {
    let temp_dir = setup_test_dir_structure();
    let data_dir = temp_dir.path();

    // Search for something completely different
    let results = find_similar_repos(data_dir, "completely/different-repository-name");

    // Should return empty or no matches based on the distance threshold
    // With current logic (distance <= 3 OR similarity >= 70%),
    // "completely/different-repository-name" is too different from existing repos
    assert!(results.is_empty());
}

#[test]
fn test_empty_data_directory() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    let results = find_similar_repos(data_dir, "user/repo");

    assert!(results.is_empty());
}

#[test]
fn test_nonexistent_data_directory() {
    let data_dir = PathBuf::from("/nonexistent/path/that/does/not/exist");

    let results = find_similar_repos(&data_dir, "user/repo");

    assert!(results.is_empty());
}

#[test]
fn test_multiple_users_same_repo_name() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    fs::create_dir_all(data_dir.join("user1/myrepo")).unwrap();
    fs::create_dir_all(data_dir.join("user2/myrepo")).unwrap();
    fs::create_dir_all(data_dir.join("user3/myrepo")).unwrap();

    let results = find_similar_repos(data_dir, "user1/myrepo");

    // All three should be found as they're all similar
    assert!(results.contains(&"user1/myrepo".to_string()));
    assert!(results.contains(&"user2/myrepo".to_string()));
    assert!(results.contains(&"user3/myrepo".to_string()));

    // user1/myrepo should be first (exact match)
    assert_eq!(results[0], "user1/myrepo");
}

#[test]
fn test_special_characters_in_repo_name() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    fs::create_dir_all(data_dir.join("user/repo-name_v2.0")).unwrap();
    fs::create_dir_all(data_dir.join("user/repo-name_v2.1")).unwrap();

    let results = find_similar_repos(data_dir, "user/repo-name_v2.0");

    assert!(results.contains(&"user/repo-name_v2.0".to_string()));
    assert!(results.contains(&"user/repo-name_v2.1".to_string()));
    // Exact match should be first
    assert_eq!(results[0], "user/repo-name_v2.0");
}

#[test]
fn test_hyphen_vs_underscore() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    fs::create_dir_all(data_dir.join("user/my-repo")).unwrap();
    fs::create_dir_all(data_dir.join("user/my_repo")).unwrap();

    let results = find_similar_repos(data_dir, "user/my-repo");

    // Both should be found (distance of 1)
    assert!(results.contains(&"user/my-repo".to_string()));
    assert!(results.contains(&"user/my_repo".to_string()));
    // Exact match first
    assert_eq!(results[0], "user/my-repo");
}

#[test]
fn test_short_repo_names() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    fs::create_dir_all(data_dir.join("u/a")).unwrap();
    fs::create_dir_all(data_dir.join("u/b")).unwrap();
    fs::create_dir_all(data_dir.join("u/c")).unwrap();

    let results = find_similar_repos(data_dir, "u/a");

    // All should be found due to distance <= 3 threshold
    assert!(results.contains(&"u/a".to_string()));
    assert!(results.contains(&"u/b".to_string()));
    assert!(results.contains(&"u/c".to_string()));
}

#[test]
fn test_long_repo_names() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    let long_name = "verylongusername/very-long-repository-name-with-many-words";
    let similar_long = "verylongusername/very-long-repository-name-with-manu-words"; // typo: manu

    fs::create_dir_all(data_dir.join(long_name)).unwrap();
    fs::create_dir_all(data_dir.join(similar_long)).unwrap();

    let results = find_similar_repos(data_dir, long_name);

    assert!(results.contains(&long_name.to_string()));
    assert!(results.contains(&similar_long.to_string()));
    assert_eq!(results[0], long_name);
}

#[test]
fn test_only_files_no_directories() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    // Create user directory but put files instead of repo directories
    fs::create_dir_all(data_dir.join("user")).unwrap();
    fs::write(data_dir.join("user/file1.txt"), "content").unwrap();
    fs::write(data_dir.join("user/file2.txt"), "content").unwrap();

    let results = find_similar_repos(data_dir, "user/repo");

    // Should return empty since there are no repo directories
    assert!(results.is_empty());
}

#[test]
fn test_nested_structure_only_two_levels() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    // Create deeper nested structure
    fs::create_dir_all(data_dir.join("user/repo/subdir")).unwrap();

    let results = find_similar_repos(data_dir, "user/repo");

    // Should find user/repo but not the subdir
    assert!(results.contains(&"user/repo".to_string()));
    assert!(!results.iter().any(|r| r.contains("subdir")));
}

#[test]
fn test_username_typo() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    fs::create_dir_all(data_dir.join("user1/myrepo")).unwrap();
    fs::create_dir_all(data_dir.join("user2/myrepo")).unwrap();

    // Search with typo in username
    let results = find_similar_repos(data_dir, "user3/myrepo");

    // Both user1/myrepo and user2/myrepo should be found (distance of 1)
    assert!(results.contains(&"user1/myrepo".to_string()));
    assert!(results.contains(&"user2/myrepo".to_string()));
}

#[test]
fn test_results_are_unique() {
    let temp_dir = setup_test_dir_structure();
    let data_dir = temp_dir.path();

    let results = find_similar_repos(data_dir, "user1/repo1");

    // Check that there are no duplicate entries
    let mut unique_results = results.clone();
    unique_results.sort();
    unique_results.dedup();

    assert_eq!(results.len(), unique_results.len());
}

#[test]
fn test_empty_target_repo() {
    let temp_dir = setup_test_dir_structure();
    let data_dir = temp_dir.path();

    let results = find_similar_repos(data_dir, "");

    // All repos should match with empty string based on similarity threshold
    // Empty string has distance equal to the length of each repo name
    // For short repo names like "user1/repo1" (11 chars), the percentage similarity
    // won't meet the threshold, so results should be empty
    assert!(results.is_empty());
}

#[test]
fn test_github_style_repo_names() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    fs::create_dir_all(data_dir.join("facebook/react")).unwrap();
    fs::create_dir_all(data_dir.join("facebook/reac")).unwrap();
    fs::create_dir_all(data_dir.join("vercel/next.js")).unwrap();

    let results = find_similar_repos(data_dir, "facebook/react");

    // Should find exact match and close typo (1 char distance)
    assert!(results.contains(&"facebook/react".to_string()));
    assert!(results.contains(&"facebook/reac".to_string()));
    assert_eq!(results[0], "facebook/react");

    // vercel/next.js should not be found (too different)
    assert!(!results.contains(&"vercel/next.js".to_string()));
}

#[test]
fn test_sorting_order_by_distance() {
    let temp_dir = TempDir::new().unwrap();
    let data_dir = temp_dir.path();

    // Create repos with varying distances from "user/test"
    fs::create_dir_all(data_dir.join("user/test")).unwrap(); // distance 0
    fs::create_dir_all(data_dir.join("user/tast")).unwrap(); // distance 1
    fs::create_dir_all(data_dir.join("user/tost")).unwrap(); // distance 1
    fs::create_dir_all(data_dir.join("user/toast")).unwrap(); // distance 2

    let results = find_similar_repos(data_dir, "user/test");

    // First should be exact match
    assert_eq!(results[0], "user/test");
    // Next two should have distance 1 (order may vary)
    assert!(results[1] == "user/tast" || results[1] == "user/tost");
    // Last should have distance 2
    assert!(results.contains(&"user/toast".to_string()));
}
