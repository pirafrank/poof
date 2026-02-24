//! File path utilities and fuzzy repository name matching.

use crate::constants::FILENAME_SEPARATORS;
use crate::constants::SUPPORTED_EXTENSIONS;
use crate::utils::string::levenshtein_distance;
use std::{
    ffi::{OsStr, OsString},
    path::Path,
};

/// Return the file extension of `archive_path` as a string slice.
///
/// Multi-part extensions such as `.tar.gz`, `.tar.xz`, `.tar.bz2`, and
/// `.tar.zst` are returned whole. For all other paths the standard
/// single-component extension is returned.
pub fn get_file_extension(archive_path: &Path) -> &str {
    let filename = archive_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default()
        .to_lowercase();

    // Handle multi-part extensions like .tar.gz, .tar.xz, .tar.bz2
    if filename.ends_with(".tar.gz") {
        return "tar.gz";
    } else if filename.ends_with(".tar.xz") {
        return "tar.xz";
    } else if filename.ends_with(".tar.bz2") {
        return "tar.bz2";
    } else if filename.ends_with(".tar.zst") {
        return "tar.zst";
    }

    // For single extensions, use the standard method
    archive_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
}

/// Return the file name component of `archive_path` as a string slice.
///
/// Returns an empty string when the path has no file-name component or when
/// the name contains non-UTF-8 bytes.
pub fn get_file_name(archive_path: &Path) -> &str {
    archive_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default()
}

/// Strip any recognised archive or compression extension from the file name of `path`.
///
/// If no known extension matches the file name is returned as-is (falling back
/// to the file stem when the path has one).
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

/// Return the "stem" of `file_name` trimmed at the first [`FILENAME_SEPARATORS`] character.
///
/// For example, `mytool-1.0.0-linux-x86_64` becomes `mytool`. Trailing ASCII
/// digits are also removed after the trim. This is used when an asset is a
/// raw executable (not an archive) so that the installed binary gets a clean,
/// version-agnostic name.
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
pub fn find_similar_repos(data_dir: &Path, target_repo: &str) -> Vec<String> {
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
    similar_repos
}

/// Find a similar repo name in the data directory based on fuzzy matching.
/// Returns the most similar repo name if found, otherwise None.
/// Never returns an empty string.
pub fn find_similar_repo(data_dir: &Path, target_repo: &str) -> Option<String> {
    let similar_repos: Vec<String> = find_similar_repos(data_dir, target_repo);
    // Return only the top entry as a string
    similar_repos.into_iter().next()
}

#[cfg(test)]
mod tests;
