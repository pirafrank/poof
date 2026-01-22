//! Unit tests for the install command module
//! Tests focus on internal logic without external dependencies

//use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Import the parent module to access functions
use crate::commands::install::*;

use anyhow::Result;

/// Helper to set up a test environment with temporary directories
struct TestEnv {
    _temp_dir: TempDir,
    home_dir: PathBuf,
}

impl TestEnv {
    fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let home = temp_dir.path().canonicalize().unwrap().to_path_buf();

        Ok(Self {
            _temp_dir: temp_dir,
            home_dir: home,
        })
    }

    fn create_dir(&self, name: &str) -> Result<PathBuf> {
        let path = self.home_dir.join(name);
        fs::create_dir_all(&path)?;
        Ok(path.canonicalize()?)
    }
}

// =============================================================================
// Tests for clean_cache_dir
// =============================================================================

#[cfg(test)]
mod clean_cache_dir_tests {
    use super::*;

    #[test]
    fn test_clean_cache_dir_success() -> Result<()> {
        let env = TestEnv::new()?;
        let cache_root = env.create_dir("cache")?;
        let cache_subdir = cache_root.join("subdir");
        fs::create_dir_all(&cache_subdir)?;

        // Add a file to the cache
        fs::write(cache_subdir.join("cached_file"), b"data")?;

        let result = clean_cache_dir(&cache_subdir, &cache_root)?;
        assert!(result, "Should return true when directory is deleted");
        assert!(
            !cache_subdir.exists(),
            "Cache subdirectory should be deleted"
        );

        Ok(())
    }

    #[test]
    fn test_clean_cache_dir_outside_root() -> Result<()> {
        let env = TestEnv::new()?;
        let cache_root = env.create_dir("cache")?;
        let outside_dir = env.create_dir("not_cache")?;

        let result = clean_cache_dir(&outside_dir, &cache_root)?;
        assert!(
            !result,
            "Should return false and refuse to delete outside cache"
        );
        assert!(
            outside_dir.exists(),
            "Directory outside cache should not be deleted"
        );

        Ok(())
    }

    #[test]
    fn test_clean_cache_dir_nonexistent() -> Result<()> {
        let env = TestEnv::new()?;
        let cache_root = env.create_dir("cache")?;
        let nonexistent = cache_root.join("does_not_exist");

        let result = clean_cache_dir(&nonexistent, &cache_root)?;
        assert!(!result, "Should return false when directory doesn't exist");

        Ok(())
    }

    #[test]
    fn test_clean_cache_dir_nested() -> Result<()> {
        let env = TestEnv::new()?;
        let cache_root = env.create_dir("cache")?;
        let nested_dir = cache_root.join("level1").join("level2").join("level3");
        fs::create_dir_all(&nested_dir)?;

        let result = clean_cache_dir(&nested_dir, &cache_root)?;
        assert!(result, "Should successfully delete nested directory");
        assert!(!nested_dir.exists(), "Nested directory should be deleted");

        Ok(())
    }
}
