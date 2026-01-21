use crate::{
    commands::{self, download::download_asset, list::list_installed_spells},
    constants::APP_NAME,
    files::{
        archives::extract_to_dir, filesys::find_exec_files_from_extracted_archive,
        magic::is_exec_by_magic_number, utils::get_stem_name_trimmed_at_first_separator,
    },
    github::client::{get_assets, get_release},
    models::spell::Spell,
    utils::semver::{SemverStringPrefix, Version},
    UpdateArgs,
};
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, error, info};
use rayon::prelude::*;

// updating a single repository
fn update_single_repo(repo: &str) -> Result<()> {
    info!("Checking for updates for {}", repo);

    // 1. get all installed assets
    let installed_assets: Vec<Spell> = list_installed_spells();

    if installed_assets.is_empty() {
        info!("No binaries installed yet. Nothing to update.");
        return Ok(());
    }

    // find the specific asset for the requested repo
    let target_asset = installed_assets
        .iter()
        .find(|asset: &&Spell| asset.get_name() == repo);

    // handle the None case first by returning early
    let Some(asset) = target_asset else {
        info!(
            "{} is not installed. Use 'poof install {}' first.",
            repo, repo
        );
        return Ok(()); // nothing to update, not an error
    };

    // we know asset exists, extract the latest version string using ?
    let highest_installed_str = asset.get_latest_version().ok_or_else(|| {
        anyhow!(
            "Spell {} found but has no versions listed (internal error)",
            repo
        )
    })?;

    let highest_installed = Version::parse(&highest_installed_str).with_context(|| {
        format!(
            "Failed to parse highest installed version '{}' as semver",
            highest_installed_str
        )
    })?;

    debug!(
        "Highest installed version for {}: {}",
        repo, highest_installed
    );

    // 2. get the latest release tag from GitHub
    // TODO: refactor get_release to return Result
    let latest_release = get_release(repo, None) // None fetches the latest release
        .with_context(|| format!("Failed to get latest release information for {}", repo))?;
    let latest_version_str = latest_release.tag_name();
    let latest_version =
        Version::parse(latest_version_str.strip_v().as_str()).with_context(|| {
            format!(
                "Failed to parse latest release tag '{}' as semver",
                latest_version_str
            )
        })?;

    info!("Latest available version for {}: {}", repo, latest_version);

    // 3. compare latest release tag with the highest installed version
    if latest_version > highest_installed {
        info!(
            "Newer version {} found for {}. Updating from {}.",
            latest_version, repo, highest_installed
        );
        // 4. call process_install for the latest tag
        commands::install::install(repo, Some(latest_version_str)).with_context(|| {
            format!(
                "Failed to install version {} as the default for {}",
                latest_version_str, repo
            )
        })?;
        info!(
            "Successfully updated {} to version {} and set it as default",
            repo, latest_version
        );
    } else {
        // 5. if no newer version, inform the user.
        info!(
            "{} is already up-to-date (version {}).",
            repo, highest_installed
        );
    }

    Ok(())
}

fn update_all_repos() -> Result<()> {
    info!("Checking for updates for all installed binaries...");

    // 1. get all installed assets
    let installed_assets: Vec<Spell> = list_installed_spells();

    if installed_assets.is_empty() {
        info!("No binaries installed yet. Nothing to update.");
        return Ok(());
    }

    info!(
        "Found {} installed repositories. Checking updates...",
        installed_assets.len()
    );

    // 2. Use rayon::par_iter to parallelize calls to update_single_repo
    let results: Vec<Result<()>> = installed_assets
        .par_iter() // parallel iterator
        .map(|asset| {
            // extract repo name for the call
            let repo_name = asset.get_name();
            // call update_single_repo for each asset
            update_single_repo(repo_name)
                // add context specific to this repo in case of failure
                .with_context(|| format!("Failed to update {}", repo_name))
        })
        .collect(); // collect results

    // 3. Collect results and report overall success/failures.
    let mut failures = Vec::new();
    for (index, result) in results.iter().enumerate() {
        if let Err(e) = result {
            // store the error along with the repo name it occurred for
            let repo_name = installed_assets[index].get_name();
            // use the error's context chain provided by anyhow
            error!("Update failed for {}: {:?}", repo_name, e);
            failures.push(format!("{}: {}", repo_name, e)); // store formatted error
        }
    }

    if failures.is_empty() {
        info!("All installed binaries checked successfully.");
        Ok(()) // return Ok from the function here
    } else {
        error!("{} repositories failed to update.", failures.len());
        bail!(
            "Update --all finished with errors:\n - {}",
            failures.join("\n - ")
        )
    }
}

/// Update poof itself
fn update_self() -> Result<()> {
    info!("Checking github.com for updates...");

    let current_version = env!("CARGO_PKG_VERSION");
    let repo = "pirafrank/poof";

    // Get the latest release from GitHub
    let latest_release = get_release(repo, None)
        .with_context(|| "Failed to get latest release information for poof")?;
    let latest_version_str = latest_release.tag_name();
    let latest_version =
        Version::parse(latest_version_str.strip_v().as_str()).with_context(|| {
            format!(
                "Failed to parse latest release tag '{}' as semver",
                latest_version_str
            )
        })?;

    let current_version_parsed = Version::parse(current_version).with_context(|| {
        format!(
            "Failed to parse current version '{}' as semver",
            current_version
        )
    })?;

    info!("Current installed version: {}", current_version);

    // Check if update is needed
    if latest_version <= current_version_parsed {
        info!(
            "{} is already up-to-date (version {}).",
            APP_NAME, current_version
        );
        return Ok(());
    }

    info!(
        "Newer version {} found. Updating from {}.",
        latest_version, current_version
    );

    // Find compatible asset
    let assets = get_assets(&latest_release).with_context(|| {
        format!(
            "Failed to find compatible asset for release {}",
            latest_version_str
        )
    })?;
    // for self-update, we only need the first asset since we are pretty sure for poof there only will be one.
    let binary = assets
        .first()
        .ok_or_else(|| anyhow!("No compatible asset found"))?;

    // Create a temporary directory for downloading
    let temp_dir = std::env::temp_dir().join(format!("poof-update-{}", latest_version_str));
    std::fs::create_dir_all(&temp_dir).with_context(|| {
        format!(
            "Failed to create temporary directory {}",
            temp_dir.display()
        )
    })?;

    // Download the binary
    download_asset(binary.name(), binary.browser_download_url(), &temp_dir).with_context(|| {
        format!(
            "Failed to download binary for version {}",
            latest_version_str
        )
    })?;

    let downloaded_file = temp_dir.join(binary.name());
    let new_binary_path = if is_exec_by_magic_number(&downloaded_file) {
        // Direct executable binary
        debug!("Downloaded file {} is an executable binary.", binary.name());
        downloaded_file
    } else {
        // Archive - extract and find the binary
        debug!(
            "Downloaded file {} is an archive. Extracting...",
            binary.name()
        );
        extract_to_dir(&downloaded_file, &temp_dir)
            .map_err(|e| anyhow!("Failed to extract archive: {}", e))?;

        let exec_files = find_exec_files_from_extracted_archive(&downloaded_file);
        if exec_files.is_empty() {
            bail!("No executable found in extracted archive");
        }

        // Find the binary matching APP_NAME or use the first executable
        let target_binary = exec_files
            .iter()
            .find(|path| {
                path.file_name()
                    .map(|n| {
                        let stem = get_stem_name_trimmed_at_first_separator(n);
                        stem.to_string_lossy() == APP_NAME || n.to_string_lossy() == APP_NAME
                    })
                    .unwrap_or(false)
            })
            .or_else(|| exec_files.first())
            .ok_or_else(|| anyhow!("No executable found in archive"))?;

        target_binary.clone()
    };

    // Use self_replace to replace the current executable
    info!("Replacing current executable with new version...");
    self_replace::self_replace(&new_binary_path).with_context(|| {
        format!(
            "Failed to replace executable with {}",
            new_binary_path.display()
        )
    })?;

    // Clean up temporary directory
    if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
        debug!(
            "Failed to clean up temporary directory {}: {}",
            temp_dir.display(),
            e
        );
    }

    info!(
        "Successfully updated {} from version {} to {}",
        APP_NAME, current_version, latest_version
    );
    info!("Please restart the application if it hasn't exited automatically.");

    Ok(())
}

// Main process
pub fn process_update(args: &UpdateArgs) -> Result<()> {
    if args.all {
        update_all_repos().context("Failed during update --all")?;
        Ok(())
    } else if args.update_self {
        update_self().context("Failed during update --self")?;
        Ok(())
    } else if let Some(repo) = &args.repo {
        update_single_repo(repo)
    } else {
        bail!("No repository specified, and neither --all nor --self flags were provided.");
    }
}
