//! Main file handling 'which' command

use anyhow::{bail, Context, Result};
use log::{debug, error};
use std::path::Path;

use crate::cli::WhichArgs;
use crate::files::datadirs;
use crate::models::slug::Slug;
use crate::output;

pub fn run_which(args: &WhichArgs) -> Result<()> {
    let bin_dir = datadirs::get_bin_dir().context("Cannot get bin directory path")?;
    let binary_path = bin_dir.join(&args.binary_name);

    // Check if binary exists
    if !binary_path.exists() {
        error!("'{}' not found in poof's bin directory.", args.binary_name);
        return Ok(());
    }

    // Try to read the symlink
    let symlink_target = match binary_path.read_link() {
        Ok(target) => target,
        Err(_) => {
            error!(
                "'{}' exists in poof's bin directory but is not a symlink.\n\
                This is likely a foreign binary not managed by poof. Please remove it and try again.",
                args.binary_name
            );
            return Ok(());
        }
    };

    // Extract the slug from the symlink target path
    let (slug, version) = extract_slug_from_path(&symlink_target).with_context(|| {
        format!(
            "Cannot determine repository providing '{}'.",
            args.binary_name
        )
    })?;

    output!("{} is provided by:", args.binary_name);
    output!("{} {}", slug, version);

    Ok(())
}

/// Extract the repository slug (username/reponame) from a binary path.
/// The path is expected to be in the format:
/// ...data_dir/SERVER_NAME/USERNAME/REPO_NAME/VERSION/binary_name
fn extract_slug_from_path(path: &Path) -> Result<(String, String)> {
    let path_str = path.to_string_lossy();

    // Find the data dir part and extract what comes after.
    // IMPORTANT: Do NOT rely on any platform-specific path for data directory.
    //            Instead, start from the data subdirectory, which is always the same.
    let marker = "poof/data/";
    if let Some(pos) = path_str.find(marker) {
        let after_data_dir = &path_str[pos + marker.len()..];

        // Split by path separator and take the first two components (username/reponame)
        let components: Vec<&str> = after_data_dir
            .split(std::path::MAIN_SEPARATOR)
            .filter(|s| !s.is_empty())
            .collect();

        // The path is expected to be in the format:
        // ...data_dir/SERVER_NAME/USERNAME/REPO_NAME/VERSION/binary_name
        if components.len() == 5 {
            return Ok((
                Slug::from_parts(components[1], components[2]).map(|slug| slug.to_string())?,
                components[3].to_string(),
            ));
        }
        bail!("Internal error");
    }

    debug!(
        "Cannot determine repository providing '{}'.",
        path.to_string_lossy()
    );
    bail!("Internal error");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_slug_from_path() {
        // note: no need to make this cross-platform since the implementation
        // does not rely on any platform-specific functionality and it starts from
        // data subdirectory wherever it is located.
        let path = Path::new(
            "/home/user/.local/share/poof/data/github.com/username/reponame/1.0.0/binary_name",
        );
        let (slug, version) = extract_slug_from_path(path).unwrap();
        assert_eq!(slug, "username/reponame");
        assert_eq!(version, "1.0.0");
    }
}
