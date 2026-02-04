use crate::cli::UpdateArgs;
use crate::commands::list::list_installed_versions_per_slug;
use crate::models::slug::Slug;
use crate::{
    commands::{self, list::list_installed_spells},
    github::client::get_release,
    models::spell::Spell,
    utils::semver::{SemverStringPrefix, Version},
};
use anyhow::{anyhow, bail, Context, Result};
use log::{debug, error, info};
use rayon::prelude::*;

// updating a single repository
fn update_single_repo(repo: &str) -> Result<()> {
    info!("Checking for updates for {}", repo);

    // 1. find the specific asset for the requested repo
    let asset = list_installed_versions_per_slug(&Slug::new(repo)?)?;

    // we know asset exists, extract the latest version string using ?
    let highest_installed_str = asset.get_latest_version().ok_or_else(|| {
        anyhow!(
            "Spell {} found but has no versions listed (internal error)",
            repo
        )
    })?;

    let highest_installed = Version::parse(&highest_installed_str).with_context(|| {
        format!(
            "Cannot parse highest installed version '{}' as semver",
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
        .with_context(|| format!("Cannot get latest release information for {}", repo))?;
    let latest_version_str = latest_release.tag_name();
    let latest_version =
        Version::parse(latest_version_str.strip_v().as_str()).with_context(|| {
            format!(
                "Cannot parse latest release tag '{}' as semver",
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
                "Cannot install version {} as the default for {}",
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
                .with_context(|| format!("Cannot update {}", repo_name))
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

// Main process
pub fn process_update(args: &UpdateArgs) -> Result<()> {
    if args.all {
        update_all_repos().context("Failed during update --all")?;
        Ok(())
    } else if let Some(repo) = &args.repo {
        update_single_repo(repo)
    } else {
        bail!("No repository specified, and --all flag was not provided.");
    }
}

#[cfg(test)]
mod tests;
