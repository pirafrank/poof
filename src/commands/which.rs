//! Main file handling 'which' command

use anyhow::{Context, Result};
use log::error;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::WhichArgs;
use crate::files::{datadirs, filesys};
use crate::output;

/// Represents a match for a binary in the data directory
#[derive(Debug, Clone)]
struct BinaryMatch {
    slug: String,
    version: String,
    #[allow(dead_code)]
    path: PathBuf,
}

pub fn run_which(args: &WhichArgs) -> Result<()> {
    let data_dir = datadirs::get_data_dir().context("Cannot get data directory path")?;

    // Find all binaries matching the requested name across all installed repositories
    let matches = find_binary_in_data_dir(&data_dir, &args.binary_name);

    if matches.is_empty() {
        error!(
            "'{}' not found in any installed repositories.",
            args.binary_name
        );
        return Ok(());
    }

    // Display results
    output!("{} is provided by:", args.binary_name);
    for m in matches {
        output!("{} {}", m.slug, m.version);
    }

    Ok(())
}

/// Search through the data directory to find all binaries matching the given name.
/// Returns a vector of BinaryMatch structs containing slug, version, and path.
fn find_binary_in_data_dir(data_dir: &Path, binary_name: &str) -> Vec<BinaryMatch> {
    // Traverse the data directory structure:
    // data_dir/github.com/username/reponame/version/
    // Using parallel iteration for performance, similar to list.rs

    let entries = match fs::read_dir(data_dir) {
        Ok(entries) => entries.flatten().collect::<Vec<_>>(),
        Err(_) => return Vec::new(),
    };

    entries
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
                            if version_path.is_dir() {
                                let version_name =
                                    version.file_name().into_string().unwrap_or_default();

                                // Find all executables in this version directory
                                let executables = filesys::find_exec_files_in_dir(&version_path);

                                // Check if any executable matches the binary name
                                executables.into_iter().find_map(|exec_path| {
                                    let file_name = exec_path.file_name()?;
                                    if file_name == binary_name {
                                        Some(BinaryMatch {
                                            slug: slug.clone(),
                                            version: version_name.clone(),
                                            path: exec_path,
                                        })
                                    } else {
                                        None
                                    }
                                })
                            } else {
                                None
                            }
                        })
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_find_binary_in_data_dir_empty() {
        // Test with a non-existent directory
        let temp_dir = std::env::temp_dir().join("poof_test_which_empty");
        let _ = fs::create_dir_all(&temp_dir);

        let matches = find_binary_in_data_dir(&temp_dir, "some_binary");
        assert!(matches.is_empty());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
