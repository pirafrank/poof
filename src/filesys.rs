use std::{io::Read, path::{Path, PathBuf}};

use poof::SUPPORTED_EXTENSIONS;

// ~/.config/APPNAME/config.json
pub fn _get_config_dir() -> Option<PathBuf> {
    if let Some(app_name) = option_env!("CARGO_PKG_NAME") {
        let home_dir = dirs::home_dir()?;
        let config_dir = home_dir.join(".config").join(app_name);
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).ok()?;
        }
        Some(config_dir)
    } else {
        None
    }
}

// ~/.local/share/APPNAME
pub fn get_data_dir() -> Option<PathBuf> {
    if let Some(app_name) = option_env!("CARGO_PKG_NAME") {
        let home_dir = dirs::home_dir()?;
        let data_dir = home_dir.join(".local").join("share").join(app_name);
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).ok()?;
        }
        Some(data_dir)
    } else {
        None
    }
}

pub fn get_bin_dir() -> Option<PathBuf> {
    let bin_dir = get_data_dir().unwrap().join("bin");
    if !bin_dir.exists() {
        std::fs::create_dir_all(&bin_dir).ok()?;
    }
    Some(bin_dir)
}

// ~/.cache/APPNAME
pub fn get_cache_dir() -> Option<PathBuf> {
    if let Some(app_name) = option_env!("CARGO_PKG_NAME") {
        let home_dir = dirs::home_dir()?;
        let cache_dir = home_dir.join(".cache").join(app_name);
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).ok()?;
        }
        Some(cache_dir)
    } else {
        None
    }
}

fn read_file_magic_number(path: &PathBuf) -> Option<Vec<u8>> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut buffer = vec![0; 4];
    if file.read_exact(&mut buffer).is_ok() {
        Some(buffer)
    } else {
        None
    }
}

fn is_exec_by_magic_number(path: &PathBuf) -> bool {
    if let Some(magic_number) = read_file_magic_number(path) {
        // Check for common executable magic numbers
        return magic_number == [0x7F, 0x45, 0x4C, 0x46] // ELF (Linux)
            || magic_number == [0x4D, 0x5A, 0x90, 0x00] // PE (Win)
            || magic_number == [0xCA, 0xFE, 0xBA, 0xBE]; // Mach-O (macOS)
    }
    false
}

fn find_exec_files_in_dir(dir: &PathBuf) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                // check criteria to determine if a file is a binary
                // 1. Check if the file is a regular file
                // 2. Check if the file is an executable by checking the magic number
                if file_type.is_file() && is_exec_by_magic_number(&entry.path()) {
                    result.push(entry.path());
                }
            }
        }
    }
    result
}

fn strip_supported_extensions(path: &Path) -> &str {
    let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or_default();
    for ext in &(SUPPORTED_EXTENSIONS) {
        if filename.ends_with(ext) {
            return &filename[..filename.len() - ext.len()];
        }
    }
    // fallback
    path.file_stem().and_then(|s| s.to_str()).unwrap_or(filename)
}

pub fn find_exec_files_from_extracted_archive(archive_path: &PathBuf) -> Vec<PathBuf> {
    let archive_parent = archive_path.parent().unwrap();
    // Get the filename without the extension
    // and create the path of a directory with the same name as the archive, minus the extension.
    // If it exists, we will search for executables in that directory.
    // If it doesn't exist, we will search for executables in the parent directory.
    // This is useful for archives that contain a directory with the same name as the archive.
    let filename_no_ext_str = strip_supported_extensions(&archive_path);
    let dir = archive_parent.join(filename_no_ext_str);
    if dir.exists() {
        return find_exec_files_in_dir(&dir);
    } else {
        return find_exec_files_in_dir(&PathBuf::from(archive_parent));
    }
}

pub fn symlink(source: &PathBuf, target: &PathBuf) -> std::io::Result<()> {
    // TODO: support windows symlinks in userspace somehow, or just copy the exe file to dir in PATH!
    // On Unix-like systems, use the `ln` command to create a symbolic link
    std::os::unix::fs::symlink(source, target)?;
    Ok(())
}
