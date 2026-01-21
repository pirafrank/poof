//! Main file handling 'list' command

use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;

use crate::files::datadirs::get_data_dir;
use crate::models::spell::Spell;

pub fn list_installed_spells() -> Vec<Spell> {
    // List all files in the bin directory.
    // Making this iterative for clarity and performance,
    // data dir as a known structure with fixed number of levels.
    // we traverse the directory tree to find all installed spells
    // and their versions without needing to recursively search through
    // the entire directory structure.
    // This is a performance optimization for the case as the data directory
    // may contain a large number of directories.
    // We will use a parallel iterator (provided by the rayon crate) to
    // speed up the process. We wont' need
    // to use a mutex because each thread will be working on a different
    // directory, with data aggregated sequentially at the end.
    let data_dir: PathBuf = get_data_dir()
        .ok_or_else(|| anyhow!("Failed to get data directory"))
        .unwrap();

    // Look through each subdirectory in data_dir for any installed spells.
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

    let mut versions_map: HashMap<String, Vec<String>> = HashMap::new();
    for (slug, version) in assets {
        versions_map.entry(slug).or_default().push(version);
    }

    let mut result: Vec<Spell> = versions_map
        .into_iter()
        .map(|(slug, versions)| Spell::new_as_string(slug, versions))
        .collect();
    result.sort();
    result
}
