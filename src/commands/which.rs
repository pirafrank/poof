//! Main file handling 'which' command

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::WhichArgs;
use crate::commands::list::list_installed_spells;
use crate::files::{datadirs, magic};
use crate::models::spell::Spell;
use crate::output;

pub fn run_which(args: &WhichArgs) -> Result<()> {
    let data_dir = datadirs::get_data_dir().context("Cannot get data directory path")?;
    let spells = list_installed_spells();

    // Find all binaries matching the requested name across all installed repositories.
    let matches = find_binary_providers(&spells, &data_dir, &args.binary_name);

    if matches.is_empty() {
        return Err(anyhow!(
            "'{}' not found in any installed repositories.",
            args.binary_name
        ));
    }

    // Display results
    output!("{} is provided by:", args.binary_name);
    for (slug, version) in matches {
        output!("{} {}", slug, version);
    }

    Ok(())
}

fn find_binary_providers(
    spells: &[Spell],
    data_dir: &Path,
    binary_name: &str,
) -> Vec<(String, String)> {
    let symlink_target = resolve_managed_symlink_target(binary_name);
    let mut matches: Vec<(String, String)> = Vec::new();

    for spell in spells {
        let slug = spell.get_name().to_owned();
        let versions_nest = datadirs::get_versions_nest(data_dir, &slug);

        for version in spell.get_versions() {
            let version_str = version.to_string();
            let version_dir = versions_nest.join(&version_str);
            let has_exact_binary = has_executable_named(&version_dir, binary_name);
            let has_symlink_for_version = symlink_target
                .as_ref()
                .is_some_and(|target| target.starts_with(&version_dir));

            if has_exact_binary || has_symlink_for_version {
                matches.push((slug.clone(), version_str));
            }
        }
    }

    matches
}

fn has_executable_named(version_dir: &Path, binary_name: &str) -> bool {
    let direct_candidate = version_dir.join(binary_name);
    if is_executable_file(&direct_candidate) {
        return true;
    }

    // one day this may be useful
    #[cfg(target_os = "windows")]
    {
        let exe_candidate = version_dir.join(format!("{}.exe", binary_name));
        if is_executable_file(&exe_candidate) {
            return true;
        }
    }

    false
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file() && magic::is_exec_by_magic_number(path)
}

fn resolve_managed_symlink_target(binary_name: &str) -> Option<PathBuf> {
    let bin_dir = datadirs::get_bin_dir()?;
    let symlink_path = bin_dir.join(binary_name);
    let target = fs::read_link(&symlink_path).ok()?;
    let absolute_target = if target.is_absolute() {
        target
    } else {
        symlink_path.parent()?.join(target)
    };

    if absolute_target.exists() {
        Some(absolute_target)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_binary_providers_empty_spells() {
        let temp_dir = std::env::temp_dir().join("poof_test_which_empty");
        let spells = Vec::new();
        let matches = find_binary_providers(&spells, &temp_dir, "some_binary");
        assert!(matches.is_empty());
    }
}
