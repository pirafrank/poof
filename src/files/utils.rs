//! File path utilities and fuzzy repository name matching.

use crate::constants::SUPPORTED_EXTENSIONS;
use crate::utils::string::levenshtein_distance;
use std::path::Path;

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
#[allow(unused)]
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
#[allow(unused)]
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

/// Remove complete metadata tokens at filename separator boundaries, ignoring ASCII case.
///
/// Compound aliases such as `x86_64` and overlapping metadata spans are removed whole. Only separator
/// runs touching removed metadata are collapsed; all other characters remain unchanged.
/// If removal would erase the command entirely, return the original filename.
pub fn clean_up_filename(filename: &str, mut to_remove: Vec<String>) -> String {
    to_remove.sort_by_key(|token| std::cmp::Reverse(token.len()));
    let lower = filename.to_ascii_lowercase();
    let mut removed = vec![false; filename.len()];
    let separator = |ch: char| matches!(ch, '-' | '_' | '.');
    for token in to_remove {
        if token.is_empty() {
            continue;
        }
        for (start, matched) in lower.match_indices(&token.to_ascii_lowercase()) {
            let end = start + matched.len();
            let left = lower[..start].chars().next_back();
            let right = lower[end..].chars().next();
            if left.is_none_or(separator) && right.is_none_or(separator) {
                removed[start..end].fill(true);
            }
        }
    }

    let chars: Vec<_> = filename.char_indices().collect();
    let mut cleaned = String::new();
    let mut index = 0;
    while index < chars.len() {
        let (offset, ch) = chars[index];
        if !removed[offset] && !separator(ch) {
            cleaned.push(ch);
            index += 1;
            continue;
        }
        let start = index;
        let mut has_removed = false;
        while index < chars.len() {
            let (offset, ch) = chars[index];
            if !removed[offset] && !separator(ch) {
                break;
            }
            has_removed |= removed[offset];
            index += 1;
        }
        if has_removed {
            if !cleaned.is_empty()
                && index < chars.len()
                && let Some((_, ch)) = chars[start..index].iter().find(|(_, ch)| separator(*ch))
            {
                cleaned.push(*ch);
            }
        } else {
            cleaned.extend(chars[start..index].iter().map(|(_, ch)| ch));
        }
    }
    if cleaned.is_empty() {
        filename.to_string()
    } else {
        cleaned
    }
}

/// Returns `true` if `item` contains the alias token, ignoring case,
/// and having it surrounded by non-alphanumeric characters.
/// Returns `false` if any of the two parameters is empty.
pub fn contains_alias_token(item: &str, alias: &str) -> bool {
    if item.is_empty() || alias.is_empty() {
        return false;
    }
    let item = item.to_lowercase();
    let alias = alias.to_lowercase();
    item.match_indices(&alias).any(|(start, _)| {
        let end = start + alias.len();
        let left_ok = item[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        let right_ok = item[end..]
            .chars()
            .next()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        left_ok && right_ok
    })
}

/// Find similar repo names in the data directory based on fuzzy matching
pub fn find_similar_repos(data_dir: &Path, target_repo: &str) -> Vec<String> {
    let mut similar_repos = Vec::new();

    if let Ok(entries) = std::fs::read_dir(data_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type()
                && file_type.is_dir()
                && let Some(username) = entry.file_name().to_str()
            {
                // Check subdirectories (repos) within each user directory
                let user_dir = data_dir.join(username);
                if let Ok(repo_entries) = std::fs::read_dir(user_dir) {
                    for repo_entry in repo_entries.flatten() {
                        if let Ok(repo_file_type) = repo_entry.file_type()
                            && repo_file_type.is_dir()
                            && let Some(repo_name) = repo_entry.file_name().to_str()
                        {
                            let full_repo = format!("{}/{}", username, repo_name);

                            // Calculate similarity
                            let distance = levenshtein_distance(target_repo, &full_repo);
                            let max_len = std::cmp::max(target_repo.len(), full_repo.len());

                            // Consider repos with distance <= 3 or similarity >= 70%
                            if distance <= 3
                                || (max_len > 0 && distance as f32 / max_len as f32 <= 0.3)
                            {
                                similar_repos.push(full_repo);
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

/// Returns `true` if `item` is a checksum file.
pub fn is_checksum_file(item: &str) -> bool {
    let item = item.to_lowercase();
    item == "checksum.txt"
        || item == "checksums.txt"
        || item.ends_with(".sha256")
        || item.ends_with(".sha256sum")
        || item.ends_with(".sha1")
        || item.ends_with(".sha1sum")
        || item.ends_with(".md5")
        || item.ends_with(".md5sum")
        || item.ends_with(".sha512")
        || item.ends_with(".sha512sum")
        || item.ends_with(".crc32")
        || item.ends_with(".crc64")
        || item.ends_with(".crc")
        || item.ends_with(".sfv")
}

/// Returns `true` if `item` is a signature file.
pub fn is_signature_file(item: &str) -> bool {
    let item = item.to_lowercase();
    item.ends_with(".asc")
        || item.ends_with(".sig")
        || item.ends_with(".pem")
        || item.ends_with(".minisign")
        || item.ends_with(".pgp")
        || item.ends_with(".gpg")
}

#[cfg(test)]
mod tests;
