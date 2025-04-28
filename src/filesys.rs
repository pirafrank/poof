use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use poof::SUPPORTED_EXTENSIONS;

// Constants for magic numbers
#[cfg(target_os = "macos")]
const MACHO_MAGIC_NUMBERS: &[[u8; 4]] = &[
    [0xFE, 0xED, 0xFA, 0xCE], // Mach-O 32-bit (little-endian)
    [0xFE, 0xED, 0xFA, 0xCF], // Mach-O 64-bit (little-endian)
    [0xCE, 0xFA, 0xED, 0xFE], // Mach-O 32-bit (big-endian)
    [0xCF, 0xFA, 0xED, 0xFE], // Mach-O 64-bit (big-endian)
    [0xCA, 0xFE, 0xBA, 0xBE], // Mach-O universal ('fat') binary (little-endian)
    [0xBE, 0xBA, 0xFE, 0xCA], // Mach-O universal ('fat') binary (big-endian)
];

// ~/.config/APPNAME/config.json
pub fn _get_config_dir() -> Option<PathBuf> {
    if let Some(app_name) = option_env!("CARGO_PKG_NAME") {
        let config_dir = dirs::config_dir()?.join(app_name);
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
        let data_dir = dirs::data_dir()?.join(app_name);
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
        let cache_dir = dirs::cache_dir()?.join(app_name);
        if !cache_dir.exists() {
            std::fs::create_dir_all(&cache_dir).ok()?;
        }
        Some(cache_dir)
    } else {
        None
    }
}

#[cfg(target_os = "linux")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    // Linux expects ELF binaries
    buffer == &[0x7F, 0x45, 0x4C, 0x46] // ELF
}

#[cfg(target_os = "windows")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    // Windows expects PE binaries (MZ header).
    // Checking only the first two bytes because the other two may change,
    // as they depend on the DOS stub.
    buffer[..2] == [0x4D, 0x5A]
}

#[cfg(target_os = "macos")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    // macOS expects Mach-O formats
    MACHO_MAGIC_NUMBERS.contains(buffer)
}

#[cfg(not(target_os = "windows"))]
fn is_exec_by_magic_number(path: &PathBuf) -> bool {
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0u8; 4];
        if file.read_exact(&mut buffer).is_ok() {
            return is_exec_magic(&buffer);
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn is_exec_by_magic_number(path: &PathBuf) -> bool {
    // We need to first check the file extension for Windows binaries,
    // as it uses the PE format (MZ header) for file types other than
    // .exe (e.g. .dll, .sys, etc.).
    // Then we check the first two bytes of the .exe file because the
    // other two may change (they depend on the DOS stub).
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if extension != "exe" {
        return false;
    }
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0u8; 4];
        if file.read_exact(&mut buffer).is_ok() {
            return is_exec_magic(&buffer);
        }
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
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default();
    for ext in &(SUPPORTED_EXTENSIONS) {
        if let Some(stripped) = filename.strip_suffix(ext) {
            return stripped;
        }
    }
    // fallback
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename)
}

pub fn find_exec_files_from_extracted_archive(archive_path: &Path) -> Vec<PathBuf> {
    let archive_parent = archive_path.parent().unwrap();
    // Get the filename without the extension
    // and create the path of a directory with the same name as the archive, minus the extension.
    // If it exists, we will search for executables in that directory.
    // If it doesn't exist, we will search for executables in the parent directory.
    // This is useful for archives that contain a directory with the same name as the archive.
    let filename_no_ext_str = strip_supported_extensions(archive_path);
    let dir = archive_parent.join(filename_no_ext_str);
    if dir.exists() {
        find_exec_files_in_dir(&dir)
    } else {
        find_exec_files_in_dir(&PathBuf::from(archive_parent))
    }
}

pub fn symlink(source: &PathBuf, target: &PathBuf) -> std::io::Result<()> {
    // TODO: support windows symlinks in userspace somehow, or just copy the exe file to dir in PATH!
    // On Unix-like systems create a symbolic link to the installed binary at target.
    std::os::unix::fs::symlink(source, target)?;
    Ok(())
}
