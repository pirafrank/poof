use poof::files::utils::*;
use std::path::PathBuf;

#[test]
fn test_zip_extension() {
    let path = PathBuf::from("archive.zip");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "zip");
}

#[test]
fn test_tar_gz_extension() {
    let path = PathBuf::from("archive.tar.gz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar.gz");
}

#[test]
fn test_tgz_extension() {
    let path = PathBuf::from("archive.tgz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tgz");
}

#[test]
fn test_tar_xz_extension() {
    let path = PathBuf::from("archive.tar.xz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar.xz");
}

#[test]
fn test_txz_extension() {
    let path = PathBuf::from("archive.txz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "txz");
}

#[test]
fn test_tar_bz2_extension() {
    let path = PathBuf::from("archive.tar.bz2");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar.bz2");
}

#[test]
fn test_tbz_extension() {
    let path = PathBuf::from("archive.tbz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tbz");
}

#[test]
fn test_tbz2_extension() {
    let path = PathBuf::from("archive.tbz2");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tbz2");
}

#[test]
fn test_tar_extension() {
    let path = PathBuf::from("archive.tar");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar");
}

#[test]
fn test_gz_extension() {
    let path = PathBuf::from("archive.gz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "gz");
}

#[test]
fn test_xz_extension() {
    let path = PathBuf::from("archive.xz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "xz");
}

#[test]
fn test_bz2_extension() {
    let path = PathBuf::from("archive.bz2");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "bz2");
}

#[test]
fn test_7z_extension() {
    let path = PathBuf::from("archive.7z");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "7z");
}

#[test]
fn test_unknown_extension() {
    let path = PathBuf::from("archive.rar");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "rar");
}

#[test]
fn test_no_extension() {
    let path = PathBuf::from("archive");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "");
}

#[test]
fn test_long_path() {
    let path = PathBuf::from("/path/to/some/archive.tar.gz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar.gz");
}

#[test]
fn test_filename_with_multiple_extensions() {
    let path = PathBuf::from("archive.old.tar.gz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar.gz");
}

#[test]
fn test_hidden_file_with_extension() {
    let path = PathBuf::from(".hidden.tar.gz");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar.gz");
}

#[test]
fn test_path_with_dots_in_directory() {
    let path = PathBuf::from("/path.with/dots.in/dir/archive.zip");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "zip");
}

#[test]
fn test_unicode_filename() {
    let path = PathBuf::from("архив.zip");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "zip");
}

#[test]
fn test_very_long_filename() {
    let long_name = "a".repeat(200) + ".tar.gz";
    let path = PathBuf::from(long_name);
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar.gz");
}

#[test]
fn test_double_tar_extension() {
    let path = PathBuf::from("archive.tar.tar");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar");
}

#[test]
fn test_mixed_case_multipart_extension() {
    let path = PathBuf::from("archive.tar.GZ");
    let ext = get_file_extension(&path);
    assert_eq!(ext, "tar.gz");
}
