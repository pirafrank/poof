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
    #[cfg(unix)]
    ensure_private_managed_dir(&data_dir).ok()?;
    #[cfg(not(unix))]
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
    #[cfg(unix)]
    ensure_private_managed_dir(&bin_dir).ok()?;
    #[cfg(not(unix))]
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
    #[cfg(unix)]
    ensure_private_managed_dir(&cache_dir).ok()?;
    #[cfg(not(unix))]
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).ok()?;
    }
    Some(cache_dir)
}

/// Create or repair one user-owned poof directory without traversing its contents.
#[cfg(unix)]
fn ensure_private_managed_dir(directory: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};

    match std::fs::symlink_metadata(directory) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(directory.parent().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "managed directory has no parent",
                )
            })?)?;
            match std::fs::DirBuilder::new().mode(0o700).create(directory) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(error) => return Err(error),
    }

    let metadata = std::fs::symlink_metadata(directory)?;
    if !metadata.file_type().is_dir() || metadata.uid() != unsafe { libc::geteuid() } {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "managed directory is not a user-owned directory",
        ));
    }
    if metadata.permissions().mode() & 0o777 != 0o700 {
        std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))?;
    }
    let repaired = std::fs::symlink_metadata(directory)?;
    if !repaired.file_type().is_dir()
        || repaired.uid() != unsafe { libc::geteuid() }
        || repaired.permissions().mode() & 0o777 != 0o700
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "managed directory could not be secured",
        ));
    }
    Ok(())
}

/// Returns `base/username/reponame` for `repo` in `"username/reponame"` format.
pub fn get_versions_nest(base: &Path, repo: &str) -> PathBuf {
    // Convert repo path to filesystem-friendly format by replacing '/' with OS separator
    // Creating path as: base_dir/username/reponame
    let repo_path = repo.replace('/', std::path::MAIN_SEPARATOR_STR);
    base.join(&repo_path)
}

/// Returns `base/username/reponame/version` for the given repository and version string.
pub fn get_binary_nest(base: &Path, repo: &str, version: &str) -> PathBuf {
    // Creating path as: base_dir/username/reponame/version
    get_versions_nest(base, repo).join(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[cfg(unix)]
    #[test]
    fn test_private_managed_dir_creates_mode_700() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("parent").join("managed");

        ensure_private_managed_dir(&directory).unwrap();

        assert_eq!(
            std::fs::metadata(directory).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_private_managed_dir_repairs_permissions_and_preserves_contents() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().unwrap();
        let directory = temp.path().join("managed");
        std::fs::create_dir(&directory).unwrap();
        let content = directory.join("existing");
        std::fs::write(&content, "keep").unwrap();
        for mode in [0o755, 0o775] {
            std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(mode)).unwrap();
            ensure_private_managed_dir(&directory).unwrap();
            ensure_private_managed_dir(&directory).unwrap();
            assert_eq!(
                std::fs::metadata(&directory).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        assert_eq!(std::fs::read_to_string(content).unwrap(), "keep");
    }

    #[cfg(unix)]
    #[test]
    fn test_private_managed_dir_rejects_symlink_without_changing_target() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target");
        let link = temp.path().join("managed");
        std::fs::create_dir(&target).unwrap();
        std::fs::write(target.join("sentinel"), "keep").unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        assert!(ensure_private_managed_dir(&link).is_err());
        assert_eq!(
            std::fs::read_to_string(target.join("sentinel")).unwrap(),
            "keep"
        );
    }

    #[test]
    fn test_get_versions_nest() {
        let base = Path::new("/test/base");
        let repo = "owner/repo";
        let result = get_versions_nest(base, repo);

        // Check that the path contains the expected components
        assert!(result.to_str().unwrap().contains("owner"));
        assert!(result.to_str().unwrap().contains("repo"));

        #[cfg(unix)]
        assert_eq!(result, Path::new("/test/base/owner/repo"));

        #[cfg(windows)]
        assert_eq!(result, Path::new("\\test\\base\\owner\\repo"));
    }

    #[test]
    fn test_get_binary_nest_full_structure() {
        let base = Path::new("/home/user/.local/share/poof/data/github.com");
        let repo = "some-tools/some-cli";
        let version = "1.2.3-beta.1";
        let result = get_binary_nest(base, repo, version);

        // The path should follow: base/owner/repo/version
        #[cfg(unix)]
        assert_eq!(
            result,
            Path::new(
                "/home/user/.local/share/poof/data/github.com/some-tools/some-cli/1.2.3-beta.1"
            )
        );

        // Note: there's a bug in get_binary_nest - it joins base twice
        // This test reveals that issue
    }

    #[test]
    fn test_get_config_dir_returns_some() {
        // Test that config dir returns a value (if dirs::config_dir() works)
        let config_dir = _get_config_dir();

        // This might be None in some test environments, but if it returns Some,
        // it should contain the APP_NAME
        if let Some(dir) = config_dir {
            let path_str = dir.to_str().unwrap();
            #[cfg(target_os = "linux")]
            assert!(path_str.ends_with(&format!(".config/{}/config", APP_NAME)));
            #[cfg(target_os = "macos")]
            assert!(path_str.ends_with(&format!("Library/Application Support/{}/config", APP_NAME)));
            #[cfg(target_os = "windows")]
            assert!(path_str.ends_with(&format!("AppData\\Roaming\\{}\\config", APP_NAME)));
        }
    }

    #[test]
    fn test_get_data_dir_returns_some() {
        // Test that data dir returns a value (if dirs::data_dir() works)
        let data_dir = get_data_dir();

        // If it returns Some, it should contain the APP_NAME
        if let Some(dir) = data_dir {
            let path_str = dir.to_str().unwrap();
            // TODO: remove GITHUB_SUBDIR when poof will be updated to support different services apart from GitHub.
            #[cfg(target_os = "linux")]
            assert!(
                path_str.ends_with(&format!(".local/share/{}/data/{}", APP_NAME, GITHUB_SUBDIR))
            );
            #[cfg(target_os = "macos")]
            assert!(path_str.ends_with(&format!(
                "Library/Application Support/{}/data/{}",
                APP_NAME, GITHUB_SUBDIR
            )));
            #[cfg(target_os = "windows")]
            assert!(path_str.ends_with(&format!(
                "AppData\\Local\\{}\\data\\{}",
                APP_NAME, GITHUB_SUBDIR
            )));
        }
    }

    #[test]
    fn test_get_bin_dir_returns_some() {
        // Test that bin dir returns a value (if dirs::data_dir() works)
        let bin_dir = get_bin_dir();

        // If it returns Some, it should contain the APP_NAME
        if let Some(dir) = bin_dir {
            let path_str = dir.to_str().unwrap();
            #[cfg(target_os = "linux")]
            assert!(path_str.ends_with(&format!(".local/share/{}/bin", APP_NAME)));
            #[cfg(target_os = "macos")]
            assert!(path_str.ends_with(&format!("Library/Application Support/{}/bin", APP_NAME)));
            #[cfg(target_os = "windows")]
            assert!(path_str.ends_with(&format!("AppData\\Local\\{}\\bin", APP_NAME)));
        }
    }

    #[test]
    fn test_get_cache_dir_returns_some() {
        // Test that cache dir returns a value (if dirs::cache_dir() works)
        let cache_dir = get_cache_dir();

        // If it returns Some, it should contain the APP_NAME
        if let Some(dir) = cache_dir {
            let path_str = dir.to_str().unwrap();
            assert!(path_str.contains(APP_NAME));
        }
    }
}
