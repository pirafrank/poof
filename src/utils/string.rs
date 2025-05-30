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
