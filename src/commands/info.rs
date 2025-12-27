use crate::constants::*;
use crate::core::platform_info::*;
use crate::files::datadirs;
use anyhow::{Context, Result};
use std::io::{self, Write};

/// Print platform information useful for debug purposes.
pub fn show_info() -> Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let mut output = String::new();

    // App info
    output.push_str(&format!(
        "\n{} - {}\n{}\n",
        APP_NAME,
        DESCRIPTION,
        long_version()
    ));

    // Platform info
    output.push_str("\nPlatform Information:\n");
    output.push_str(&format!("  OS family : {}\n", std::env::consts::FAMILY));
    output.push_str(&format!("  OS type   : {}\n", std::env::consts::OS));
    output.push_str(&format!("  OS version: {}\n", get_os_version()));
    output.push_str(&format!("  Arch      : {}\n", std::env::consts::ARCH));
    output.push_str(&format!("  Endianness: {}\n", get_platform_endianness()));

    let kernel = std::process::Command::new("uname")
        .arg("-a")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| UNKNOWN.to_string());
    output.push_str(&format!("  Kernel    : {}\n", kernel));

    let executable = std::env::current_exe().unwrap_or_default();
    output.push_str(&format!("  Executable: {}\n", executable.display()));

    let cwd = std::env::current_dir().unwrap_or_default();
    output.push_str(&format!("  Cwd       : {}\n", cwd.display()));

    // Environment variables
    output.push_str("\nEnvironment:\n");
    output.push_str(&format!("  SHELL: {}\n", get_shell_info()));
    output.push_str(&format!("  USER : {}\n", get_env_var("USER")));
    output.push_str(&format!("  HOME : {}\n", get_env_var("HOME")));

    let bin_dir = datadirs::get_bin_dir().context("Failed to locate bin directory")?;
    let path_status = match check_dir_in_path(
        bin_dir
            .to_str()
            .context("Failed to convert bin directory path to string")?,
    ) {
        -1 => "Not in PATH",
        0 => "In PATH at the beginning",
        _ => "In PATH, but NOT at the beginning",
    };
    output.push_str(&format!("  PATH : {}\n", path_status));

    // Directories
    output.push_str("\nDirectories:\n");

    let cache_dir = datadirs::get_cache_dir().context("Failed to locate cache directory")?;
    output.push_str(&format!("  Cache dir: {}\n", cache_dir.display()));

    let data_dir = datadirs::get_data_dir().context("Failed to locate data directory")?;
    //TODO: remove .parent() when poof will be updated to support different services apart from GitHub.
    output.push_str(&format!(
        "  Data dir : {}\n",
        data_dir.parent().unwrap().display()
    ));

    output.push_str(&format!("  Bin dir  : {}\n", bin_dir.display()));

    // Write everything at once
    handle
        .write_all(output.as_bytes())
        .context("Failed to write output")?;
    handle.flush().context("Failed to flush output")?;

    Ok(())
}
