//! Main file handling 'check' command

use anyhow::{Context, Result};
use log::{debug, warn};
use std::path::PathBuf;

use crate::core::platform_info;
use crate::files::datadirs;

pub fn check_if_bin_in_path() -> Result<()> {
    let bin_dir: PathBuf = datadirs::get_bin_dir().context("Cannot locate bin directory")?;
    let position = platform_info::check_dir_in_path(bin_dir.to_str().unwrap());
    match position {
        -1 => {
            warn!("Bin directory not found in PATH.");
            warn!(
                "Please add {} to your PATH. For example, run: \n\n{}\n",
                bin_dir.display(),
                get_export_command()?
            );
            warn!("This is required to run the installed binaries.");
        }
        0 => debug!("Everything looks good. Bin directory is the first in PATH."),
        _ => {
            warn!("Bin directory is not the first in PATH.");
            warn!(
                "Please move {} to the beginning of your PATH.",
                bin_dir.display()
            );
        }
    }
    Ok(())
}

fn get_export_command() -> Result<String> {
    let bin_dir: PathBuf = datadirs::get_bin_dir().context("Cannot locate bin directory")?;
    Ok(format!(
        "export PATH=\"{}:$PATH\"",
        bin_dir.to_str().unwrap()
    ))
}
