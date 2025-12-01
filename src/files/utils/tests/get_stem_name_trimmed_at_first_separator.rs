use crate::files::utils::get_stem_name_trimmed_at_first_separator;
use std::ffi::OsStr;
use std::ffi::OsString;

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
