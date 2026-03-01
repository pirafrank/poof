//! Main file handling 'what' command

use anyhow::{bail, Context, Result};
use log::error;
use std::fs;

use crate::cli::WhatArgs;
use crate::files::datadirs;
use crate::files::filesys;
use crate::files::utils::find_similar_repo;
use crate::models::slug::Slug;
use crate::output;
use crate::utils::semver::SemverSort;

/// List the executables provided by the latest installed version of a repository.
///
/// Validates that `args.repo` is an installed slug, resolves the latest version
/// via semver sorting, and prints all executable files found in that version's
/// install directory.
pub fn run_what(args: &WhatArgs) -> Result<()> {
    // Validate slug
    let slug = Slug::new(&args.repo)?;

    // Get data directory
    let data_dir = datadirs::get_data_dir().context("Cannot get data directory path")?;

    // Build path to slug's versions directory
    let versions_dir = datadirs::get_versions_nest(&data_dir, slug.as_str());

    // Check if the slug is installed
    if !versions_dir.exists() {
        if let Some(similar_repo) = find_similar_repo(&data_dir, slug.as_str()) {
            error!(
                "It looks like '{}' is not installed. Did you mean: {}",
                slug, similar_repo
            );
        } else {
            error!("It looks like '{}' is not installed. Typo?", slug);
        }
        error!("Check installed binaries using 'list' command.");
        bail!("Repository '{}' not found", slug);
    }

    // Read all version subdirectories
    let entries = fs::read_dir(&versions_dir)
        .with_context(|| format!("Cannot read versions directory for '{}'", slug))?;

    let mut versions: Vec<String> = Vec::new();
    for entry in entries.flatten() {
        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                if let Some(version_name) = entry.file_name().to_str() {
                    versions.push(version_name.to_string());
                }
            }
        }
    }

    // Check if any versions were found
    if versions.is_empty() {
        error!(
            "No versions found for '{}'. Installation may be corrupted.",
            slug
        );
        bail!("No versions found for '{}'", slug);
    }

    // Sort versions using semantic versioning
    versions.sort_semver();

    // Get the latest version (last element after sorting)
    let latest_version = versions
        .last()
        .expect("versions is non-empty after check")
        .clone();

    // Build path to latest version directory
    let latest_version_dir = datadirs::get_binary_nest(&data_dir, slug.as_str(), &latest_version);

    // Find all executables in the latest version directory
    let binaries = filesys::find_exec_files_in_dir(&latest_version_dir, false);

    // Check if any binaries were found
    if binaries.is_empty() {
        error!(
            "No binaries found in version {} of '{}'. Installation may be corrupted.",
            latest_version, slug
        );
        bail!("No binaries found for '{}'", slug);
    }

    // Output the results
    output!("{} (version {}) provides:", slug, latest_version);
    for binary_path in binaries {
        if let Some(binary_name) = binary_path.file_name() {
            output!("- {}", binary_name.to_string_lossy());
        }
    }

    Ok(())
}
