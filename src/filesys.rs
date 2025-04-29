use log::debug;
use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use std::fs;

use crate::models::asset::Asset;
use crate::models::asset::VecAssets;
use poof::SUPPORTED_EXTENSIONS;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const DATA_SUBDIR: &str = "data";
const BIN_SUBDIR: &str = "bin";

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

/// This function returns the path to the config directory for the application.
/// It creates the directory if it doesn't exist.
///
/// Linux: ~/.config/APPNAME/config
///
/// macOS: ~/Library/Application Support/APPNAME/config
///
/// Windows: %APPDATA%/APPNAME/config
///
pub fn _get_config_dir() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?.join(APP_NAME).join("config");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).ok()?;
    }
    Some(config_dir)
}

/// This function returns the path to the data directory for the application.
/// It creates the directory if it doesn't exist.
///
/// Linux: $HOME/.local/share/APPNAME/data
///
/// macOS: ~/Library/Application Support/APPNAME/data
///
/// Windows: %LOCALAPPDATA%/APPNAME/data
///
pub fn get_data_dir() -> Option<PathBuf> {
    let data_dir = dirs::data_dir()?.join(APP_NAME).join(DATA_SUBDIR);
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).ok()?;
    }
    Some(data_dir)
}

/// This function returns the path to the bin directory for the application.
/// It creates the directory if it doesn't exist.
/// This is where the binaries will be stored.
///
/// Linux: ~/.local/share/APPNAME/bin
///
/// macOS: ~/Library/Application Support/APPNAME/bin
///
/// Windows: %LOCALAPPDATA%/APPNAME/bin
///
pub fn get_bin_dir() -> Option<PathBuf> {
    let bin_dir = dirs::data_dir()?.join(APP_NAME).join(BIN_SUBDIR);
    if !bin_dir.exists() {
        std::fs::create_dir_all(&bin_dir).ok()?;
    }
    Some(bin_dir)
}

/// This function returns the path to the cache directory for the application.
/// It creates the directory if it doesn't exist.
/// This is where the cache files will be stored.
///
/// Linux: ~/.cache/APPNAME
///
/// macOS: ~/Library/Caches/APPNAME
///
/// Windows: %LOCALAPPDATA%/APPNAME/cache
///
pub fn get_cache_dir() -> Option<PathBuf> {
    let cache_dir = dirs::cache_dir()?.join(APP_NAME);
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).ok()?;
    }
    Some(cache_dir)
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

#[cfg(not(target_os = "windows"))]
pub fn is_executable(path: &PathBuf) -> bool {
    // Check if the file is executable
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.is_file() {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
pub fn make_executable(installed_exec: &Path) {
    // Unix-like systems require setting executable permissions
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(installed_exec).unwrap().permissions();
    // Add executable bits to current permissions (equivalent to chmod +x)
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(installed_exec, perms).unwrap();
    debug!(
        "Set executable permissions for {}",
        installed_exec.display()
    );
}

pub fn symlink(source: &PathBuf, target: &PathBuf) -> std::io::Result<()> {
    // TODO: support windows symlinks in userspace somehow, or just copy the exe file to dir in PATH!
    // On Unix-like systems create a symbolic link to the installed binary at target.
    std::os::unix::fs::symlink(source, target)?;
    Ok(())
}

pub fn list_installed_assets() -> Vec<Asset> {
    // List all files in the bin directory.
    // Making this iterative for clarity and performance,
    // data dir as a known structure with fixed number of levels.
    // we traverse the directory tree to find all installed assets
    // and their versions without needing to recursively search through
    // the entire directory structure.
    // This is a performance optimization for the case as the data directory
    // may contain a large number of directories.
    // We will use a parallel iterator (provided by the rayon crate) to
    // speed up the process. We wont' need
    // to use a mutex because each thread will be working on a different
    // directory, with data aggregated sequentially at the end.
    let data_dir: PathBuf = get_data_dir().unwrap();

    // Look through each subdirectory in data_dir for any installed assets.
    // Read user directories in parallel.

    let entries = match fs::read_dir(&data_dir) {
        Ok(entries) => entries.flatten().collect::<Vec<_>>(),
        Err(_) => return Vec::new(),
    };

    let assets: Vec<(String, String)> = entries
        .into_par_iter()
        .filter(|user| user.path().is_dir())
        .flat_map(|user| {
            let username = user.file_name().into_string().unwrap_or_default();
            fs::read_dir(user.path())
                .ok()
                .into_iter()
                .flatten()
                .flatten()
                .filter(|repo| repo.path().is_dir())
                .flat_map(move |repo| {
                    let repo_name = repo.file_name().into_string().unwrap_or_default();
                    let slug = format!("{}/{}", username, repo_name);

                    fs::read_dir(repo.path())
                        .ok()
                        .into_iter()
                        .flatten()
                        .flatten()
                        .filter_map(move |version| {
                            let version_path = version.path();
                            if version_path.is_dir()
                                && version_path
                                    .read_dir()
                                    .map(|mut d| d.next().is_some())
                                    .unwrap_or(false)
                            {
                                let version_name =
                                    version.file_name().into_string().unwrap_or_default();
                                Some((slug.clone(), version_name))
                            } else {
                                None
                            }
                        })
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let mut map: HashMap<String, Asset> = HashMap::new();
    for (slug, version) in assets {
        // cloning here is necessary and impact is bare
        let s = slug.clone();
        map.entry(slug)
            .and_modify(|asset| asset.add_version_as_string(&version))
            .or_insert_with(|| Asset::new_as_string(s, vec![version]));
    }

    let mut result: Vec<Asset> = map.into_values().collect();
    result.sort();
    result
}
