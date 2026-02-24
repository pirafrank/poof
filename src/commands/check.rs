//! Main file handling 'check' command

use anyhow::Result;
use log::{debug, error, warn};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use crate::core::platform_info;
use crate::files::datadirs;

/// Check whether poof's bin directory is present in `PATH`.
///
/// Returns [`ExitCode::SUCCESS`] (0) when the bin directory is the first entry,
/// [`ExitCode::FAILURE`] (1) when it is present but not first, and exit code 2 when
/// it is missing entirely.
pub fn check_if_bin_in_path() -> Result<ExitCode> {
    let bin_dir: PathBuf = match datadirs::get_bin_dir() {
        Some(dir) => dir,
        None => {
            error!("Cannot locate bin directory.");
            return Ok(ExitCode::from(2u8));
        }
    };
    let position = platform_info::check_dir_in_path(bin_dir.to_str().unwrap());
    match position {
        -1 => {
            error!("Bin directory not found in PATH.");
            error!(
                "Please add {} to your PATH. For example, run: \n\n{}\n",
                bin_dir.display(),
                get_export_command(&bin_dir)?
            );
            error!("This is required to run the binaries managed by poof.");
            Ok(ExitCode::from(2u8))
        }
        0 => {
            debug!("Everything looks good. Bin directory is the first in PATH.");
            Ok(ExitCode::SUCCESS)
        }
        _ => {
            warn!("Bin directory is not the first in PATH.");
            warn!(
                "Please move {} to the beginning of your PATH.",
                bin_dir.display()
            );
            Ok(ExitCode::FAILURE)
        }
    }
}

/// Returns the shell export command that adds `bin_dir` to `PATH`.
fn get_export_command(bin_dir: &Path) -> Result<String> {
    Ok(format!("export PATH=\"{}:$PATH\"", bin_dir.display()))
}
