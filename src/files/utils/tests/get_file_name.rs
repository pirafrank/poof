use std::path::Path;

use crate::files::utils::get_file_name;

#[test]
fn test_simple_filename() {
    let path = Path::new("example.txt");
    assert_eq!(get_file_name(path), "example.txt");
}

#[test]
fn test_filename_with_path() {
    let path = Path::new("/path/to/file.txt");
    assert_eq!(get_file_name(path), "file.txt");
}

#[test]
fn test_filename_with_multiple_directories() {
    let path = Path::new("/usr/local/bin/archive.tar.gz");
    assert_eq!(get_file_name(path), "archive.tar.gz");
}

#[test]
fn test_filename_without_extension() {
    let path = Path::new("/path/to/README");
    assert_eq!(get_file_name(path), "README");
}

#[test]
fn test_filename_with_multiple_dots() {
    let path = Path::new("my.archive.file.tar.gz");
    assert_eq!(get_file_name(path), "my.archive.file.tar.gz");
}

#[test]
fn test_relative_path() {
    let path = Path::new("../relative/path/file.zip");
    assert_eq!(get_file_name(path), "file.zip");
}

#[test]
fn test_current_directory_relative() {
    let path = Path::new("./file.txt");
    assert_eq!(get_file_name(path), "file.txt");
}

#[test]
fn test_filename_with_spaces() {
    let path = Path::new("/path/to/my file with spaces.txt");
    assert_eq!(get_file_name(path), "my file with spaces.txt");
}

#[test]
fn test_filename_with_special_characters() {
    let path = Path::new("/path/to/file-name_v1.0.tar.gz");
    assert_eq!(get_file_name(path), "file-name_v1.0.tar.gz");
}

#[test]
fn test_hidden_file() {
    let path = Path::new("/path/to/.hidden");
    assert_eq!(get_file_name(path), ".hidden");
}

#[test]
fn test_hidden_file_with_extension() {
    let path = Path::new("/path/to/.hidden.txt");
    assert_eq!(get_file_name(path), ".hidden.txt");
}

#[test]
fn test_filename_only() {
    let path = Path::new("archive.zip");
    assert_eq!(get_file_name(path), "archive.zip");
}

#[test]
fn test_tar_gz_extension() {
    let path = Path::new("archive.tar.gz");
    assert_eq!(get_file_name(path), "archive.tar.gz");
}

#[test]
fn test_single_character_filename() {
    let path = Path::new("/path/a");
    assert_eq!(get_file_name(path), "a");
}

#[test]
fn test_windows_style_path() {
    let path = Path::new("C:\\Users\\Documents\\file.txt");
    // On Unix, this will be treated as a single filename
    #[cfg(unix)]
    assert_eq!(get_file_name(path), "C:\\Users\\Documents\\file.txt");
    // On Windows, this will extract just the filename
    #[cfg(windows)]
    assert_eq!(get_file_name(path), "file.txt");
}

#[test]
fn test_double_extension() {
    let path = Path::new("document.backup.zip");
    assert_eq!(get_file_name(path), "document.backup.zip");
}

#[test]
fn test_filename_starting_with_dot() {
    let path = Path::new(".gitignore");
    assert_eq!(get_file_name(path), ".gitignore");
}

#[test]
fn test_complex_path_with_dots() {
    let path = Path::new("/home/user/../other/./file.txt");
    assert_eq!(get_file_name(path), "file.txt");
}
