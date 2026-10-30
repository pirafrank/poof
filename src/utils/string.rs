//!
//! This file contains utility functions for string manipulation.
//!

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
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *cell = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

/// Strip all instances of double separators from a filename.
/// This method removed the double separators entirely, it
/// won't just collapse them into a single one.
/// This is the intended behavior.
///
/// # Arguments
///
/// * `filename` - The filename to strip double separators from.
/// * `sep` - The separator to strip.
///
/// # Returns
///
/// The filename with all instances of double separators removed.
pub fn strip_repeated_separator(filename: &str, sep: &str) -> String {
    // safe guard against empty separator
    if sep.is_empty() {
        return filename.to_string();
    }

    let mut result = filename.to_string();
    let double_sep = format!("{}{}", sep, sep);
    while result.contains(&double_sep) {
        result = result.replace(&double_sep, "");
    }
    // if an odd separator remains at the end, remove one full separator token
    if let Some(stripped) = result.strip_suffix(sep) {
        result = stripped.to_string();
    }
    result
}

/// Returns whether `part` is a safe single path component.
/// Prevent path traversal by rejecting `.`/`..` and path separators,
/// with customizable extra allowed bytes.
fn is_safe_component_with(part: &str, extra: &[u8]) -> bool {
    !part.is_empty()
        && part != "."
        && part != ".."
        && !part.contains(['/', '\\'])
        && !part.contains(':')
        && part.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'_' | b'-' | b'.')
                || extra.contains(&byte)
        })
}

/// Returns whether `part` is a safe single path component.
/// Prevent path traversal by rejecting `.`/`..` and path separators,
/// and reject path-bearing identifiers that are not safe single components.
pub fn is_safe_component(part: &str) -> bool {
    is_safe_component_with(part, &[])
}

/// Reject versions that can escape their repository directory.
pub fn validate_version(version: &str) -> anyhow::Result<()> {
    anyhow::ensure!(is_safe_component(version), "Invalid version format");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_of_str_in_string() {
        assert_eq!(position_of_str_in_string("a,b,c".to_string(), ",", "b"), 1);
        assert_eq!(position_of_str_in_string("a,b,c".to_string(), ",", "d"), -1);
        assert_eq!(position_of_str_in_string("a,b,c".to_string(), ",", "a"), 0);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(
            levenshtein_distance(
                "pirafrank/rust_exif_renamer",
                "pirafrank/rust_exit_renamere"
            ),
            2
        );
        assert_eq!(levenshtein_distance("abc", "def"), 3);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_strip_double_separator() {
        assert_eq!(strip_repeated_separator("a-b-c", "-"), "a-b-c");
        assert_eq!(strip_repeated_separator("a-b--c", "-"), "a-bc");
        assert_eq!(strip_repeated_separator("a--b--c", "-"), "abc");
        assert_eq!(strip_repeated_separator("abc---", "-"), "abc");
        assert_eq!(strip_repeated_separator("a-bc---", "-"), "a-bc");
        assert_eq!(strip_repeated_separator("a--bc---", "-"), "abc");
        assert_eq!(strip_repeated_separator("abc", "-"), "abc");
    }

    #[test]
    fn test_is_safe_component() {
        assert!(is_safe_component("valid_component"));
        assert!(!is_safe_component("valid_component@123"));
        assert!(!is_safe_component("."));
        assert!(!is_safe_component(".."));
        assert!(!is_safe_component("invalid/component"));
        assert!(!is_safe_component("invalid\\component"));
        assert!(!is_safe_component("invalid:component"));
    }

    #[test]
    fn test_is_safe_component_with() {
        assert!(is_safe_component_with("valid_component", &[]));
        assert!(!is_safe_component_with(".", &[]));
        assert!(!is_safe_component_with("..", &[]));
        assert!(!is_safe_component_with("invalid/component", &[]));
        assert!(!is_safe_component_with("invalid\\component", &[]));
        assert!(!is_safe_component_with("invalid:component", &[]));
        assert!(!is_safe_component_with("invalid:component", b"+")); // single extra byte
        assert!(!is_safe_component_with("invalid:component", b"+@")); // multiple extra bytes
        assert!(!is_safe_component_with("valid_component@123", &[])); // invalid because not allowing '@' by default
        assert!(is_safe_component_with("valid_component@123", b"+@")); // valid by adding '@' to the allowed extra bytes
    }

    #[test]
    fn test_validate_version() {
        assert!(validate_version("../../outside").is_err());
        assert!(validate_version("/outside").is_err());
        assert!(validate_version("1.2.3-beta.1").is_ok());
    }
}
