use crate::files::utils::contains_alias_token;

#[test]
fn test_contains_alias_token_exact_match() {
    assert!(contains_alias_token("linux", "linux"));
}

#[test]
fn test_contains_alias_token_at_start_with_separator_on_right() {
    assert!(contains_alias_token("linux-x86_64", "linux"));
}

#[test]
fn test_contains_alias_token_at_end_with_separator_on_left() {
    assert!(contains_alias_token("my-tool-linux", "linux"));
}

#[test]
fn test_contains_alias_token_in_middle_with_separators_on_both_sides() {
    assert!(contains_alias_token("tool-linux-amd64", "linux"));
}

#[test]
fn test_contains_alias_token_substring_of_alphanumeric_token() {
    assert!(!contains_alias_token("linuxtool", "linux"));
}

#[test]
fn test_contains_alias_token_embedded_on_right() {
    assert!(!contains_alias_token("linux64", "linux"));
}

#[test]
fn test_contains_alias_token_embedded_on_left() {
    assert!(!contains_alias_token("mylinux", "linux"));
}

#[test]
fn test_contains_alias_token_case_insensitive() {
    assert!(contains_alias_token("Linux-x86", "linux"));
}

#[test]
fn test_contains_alias_token_no_match() {
    assert!(!contains_alias_token("darwin-amd64", "linux"));
}

#[test]
fn test_contains_alias_token_empty_alias_in_word() {
    // An empty alias never satisfies the word-boundary check inside an alphanumeric token.
    assert!(!contains_alias_token("tool", ""));
}

#[test]
fn test_contains_alias_token_empty_alias_at_separator() {
    // An empty alias at a separator boundary (start of string) does satisfy the check.
    // But seriously, would a tool ever start with a separator?
    assert!(contains_alias_token("-tool", ""));
}
