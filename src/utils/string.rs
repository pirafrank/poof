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
}
