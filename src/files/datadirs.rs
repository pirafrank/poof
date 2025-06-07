use std::path::{Path, PathBuf};

use crate::constants::*;

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
    //TODO: remove .join(GITHUB_SUBDIR) when poof will be updated to support different services apart from GitHub.
    let data_dir = dirs::data_dir()?
        .join(APP_NAME)
        .join(DATA_SUBDIR)
        .join(GITHUB_SUBDIR);
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

// Function to get a path for a binary file with the directory
// structure for a specific repository and version.
pub fn get_versions_nest(base: &Path, repo: &str) -> PathBuf {
    // Convert repo path to filesystem-friendly format by replacing '/' with OS separator
    // Creating path as: base_dir/username/reponame
    let repo_path = repo.replace('/', std::path::MAIN_SEPARATOR_STR);
    base.join(&repo_path)
}

// Function to get a path for a binary file with the directory
// structure for a specific repository and version.
pub fn get_binary_nest(base: &Path, repo: &str, version: &str) -> PathBuf {
    // Creating path as: base_dir/username/reponame/version
    base.join(get_versions_nest(base, repo)).join(version)
}
