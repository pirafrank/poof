use crate::constants::FILENAME_SEPARATORS;
use crate::constants::SUPPORTED_EXTENSIONS;
use crate::utils::string::levenshtein_distance;
use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

pub fn get_file_extension(archive_path: &Path) -> &str {
    let filename = archive_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default();

    // Handle multi-part extensions like .tar.gz, .tar.xz, .tar.bz2
    if filename.ends_with(".tar.gz") {
        return "tar.gz";
    } else if filename.ends_with(".tar.xz") {
        return "tar.xz";
    } else if filename.ends_with(".tar.bz2") {
        return "tar.bz2";
    }

    // For single extensions, use the standard method
    archive_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
}

pub fn get_file_name(archive_path: &Path) -> &str {
    archive_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default()
}

pub fn strip_supported_extensions(path: &Path) -> &str {
    let filename = get_file_name(path);
    SUPPORTED_EXTENSIONS
        .iter()
        .find_map(|ext| filename.strip_suffix(ext))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(filename)
        })
}

pub fn get_stem_name_trimmed_at_first_separator(file_name: &OsStr) -> OsString {
    let x = FILENAME_SEPARATORS
        .iter()
        .fold(file_name.to_string_lossy().to_string(), |acc, sep| {
            if let Some(index) = acc.find(sep) {
                acc[..index].to_string()
            } else {
                acc
            }
        })
        .trim_end_matches(|c: char| c.is_ascii_digit())
        .to_string();
    OsString::from(x)
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
                                            let distance =
                                                levenshtein_distance(target_repo, &full_repo);
                                            let max_len =
                                                std::cmp::max(target_repo.len(), full_repo.len());

                                            // Consider repos with distance <= 3 or similarity >= 70%
                                            if distance <= 3
                                                || (max_len > 0
                                                    && distance as f32 / max_len as f32 <= 0.3)
                                            {
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
    use std::ffi::OsStr;

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_with_underscore() {
        let file_name = OsStr::new("myapp_v1.2.3_linux_amd64");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from("myapp"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_with_hyphen() {
        let file_name = OsStr::new("myapp-v1.2.3-linux-amd64");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from("myapp"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_with_dot() {
        let file_name = OsStr::new("myapp.v1.2.3.linux.amd64");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from("myapp"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_mixed_separators() {
        let file_name = OsStr::new("myapp-1.2_linux.amd64");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        // Should stop at the first separator it finds (hyphen in this case)
        assert_eq!(result, OsString::from("myapp"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_no_separators() {
        let file_name = OsStr::new("myapp");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from("myapp"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_with_trailing_digits() {
        let file_name = OsStr::new("myapp123_version");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from("myapp"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_only_digits_after_separator() {
        let file_name = OsStr::new("myapp_123456");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from("myapp"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_digits_in_name() {
        let file_name = OsStr::new("my2app_version");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        // Digits in the middle should be preserved, only trailing digits are trimmed
        assert_eq!(result, OsString::from("my2app"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_empty_string() {
        let file_name = OsStr::new("");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from(""));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_separator_at_start() {
        let file_name = OsStr::new("_myapp_version");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        // Should return empty string since separator is at the beginning
        assert_eq!(result, OsString::from(""));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_multiple_trailing_digits() {
        let file_name = OsStr::new("myapp12345_version");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from("myapp"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_real_world_examples() {
        // Test with real-world binary names
        let file_name1 = OsStr::new("ripgrep-13.0.0-x86_64-unknown-linux-musl");
        let result1 = get_stem_name_trimmed_at_first_separator(file_name1);
        assert_eq!(result1, OsString::from("ripgrep"));

        let file_name2 = OsStr::new("bat_0.24.0_amd64.deb");
        let result2 = get_stem_name_trimmed_at_first_separator(file_name2);
        assert_eq!(result2, OsString::from("bat"));

        let file_name3 = OsStr::new("fd-v8.7.0-x86_64-unknown-linux-gnu.tar.gz");
        let result3 = get_stem_name_trimmed_at_first_separator(file_name3);
        assert_eq!(result3, OsString::from("fd"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_unicode() {
        let file_name = OsStr::new("myapp_测试_version");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from("myapp"));
    }

    #[test]
    fn test_get_stem_name_trimmed_at_first_separator_only_separators() {
        let file_name = OsStr::new("___");
        let result = get_stem_name_trimmed_at_first_separator(file_name);
        assert_eq!(result, OsString::from(""));
    }
}
