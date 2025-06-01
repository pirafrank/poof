//!
//! This file contains utility functions for string manipulation.
//!

use std::path::Path;

/// Get the position of a substring in a string where the string is split by a separator.
pub fn position_of_str_in_string(input: String, sep: &str, item: &str) -> i16 {
    let mut position: i16 = 0;
    let mut found = false;
    for i in input.split(sep) {
        if i == item {
            found = true;
            break;
        }
        position += 1;
    }
    if found {
        return position;
    };
    -1 // not found
}

/// Calculate the Levenshtein distance between two strings
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1,        // deletion
                    matrix[i][j - 1] + 1         // insertion
                ),
                matrix[i - 1][j - 1] + cost      // substitution
            );
        }
    }

    matrix[len1][len2]
}

/// Find similar repo names in the data directory based on fuzzy matching
pub fn find_similar_repos(data_dir: &Path, target_repo: &str) -> String {
    let mut similar_repos = Vec::new();

    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if let Some(username) = entry.file_name().to_str() {
                        // Check subdirectories (repos) within each user directory
                        let user_dir = data_dir.join(username);
                        if let Ok(repo_entries) = std::fs::read_dir(user_dir) {
                            for repo_entry in repo_entries.flatten() {
                                if let Ok(repo_file_type) = repo_entry.file_type() {
                                    if repo_file_type.is_dir() {
                                        if let Some(repo_name) = repo_entry.file_name().to_str() {
                                            let full_repo = format!("{}/{}", username, repo_name);

                                            // Calculate similarity
                                            let distance = levenshtein_distance(target_repo, &full_repo);
                                            let max_len = std::cmp::max(target_repo.len(), full_repo.len());

                                            // Consider repos with distance <= 3 or similarity >= 70%
                                            if distance <= 3 || (max_len > 0 && distance as f32 / max_len as f32 <= 0.3) {
                                                similar_repos.push(full_repo);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by similarity (lower distance = more similar)
    similar_repos.sort_by(|a, b| {
        let dist_a = levenshtein_distance(target_repo, a);
        let dist_b = levenshtein_distance(target_repo, b);
        dist_a.cmp(&dist_b)
    });

    // Return only the top entry as a string
    similar_repos.into_iter().next().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("pirafrank/rust_exif_renamer", "pirafrank/rust_exit_renamere"), 2);
        assert_eq!(levenshtein_distance("abc", "def"), 3);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_position_of_str_in_string() {
        assert_eq!(position_of_str_in_string("a,b,c".to_string(), ",", "b"), 1);
        assert_eq!(position_of_str_in_string("a,b,c".to_string(), ",", "d"), -1);
        assert_eq!(position_of_str_in_string("a,b,c".to_string(), ",", "a"), 0);
    }
}
