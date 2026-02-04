use crate::constants::*;
use crate::core::platform_info::{long_version, short_description};
use crate::models::supported_shells::SupportedShell;

use clap::{ArgGroup, Parser, Subcommand};
use lazy_static::lazy_static;
use regex::Regex;

// Constants

lazy_static! {
    static ref REPO_REGEX: Regex = Regex::new(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$").unwrap();
    static ref BINARY_NAME_REGEX: Regex = Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
}

fn validate_repo_format(s: &str) -> Result<String, String> {
    if REPO_REGEX.is_match(s) {
        Ok(s.to_string())
    } else {
        Err(format!(
            "Repository must be in the format USERNAME/REPO, got: {}",
            s
        ))
    }
}

fn validate_binary_name(s: &str) -> Result<String, String> {
    if BINARY_NAME_REGEX.is_match(s) {
        Ok(s.to_string())
    } else {
        Err(format!(
            "Binary name must contain only letters, numbers, underscores, and hyphens, got: {}",
            s
        ))
    }
}

#[derive(Parser, Clone)]
pub struct UseArgs {
    /// GitHub user and repository in the format USERNAME/REPO
    /// e.g. pirafrank/rust_exif_renamer
    #[arg(required = true, value_parser = validate_repo_format)]
    pub repo: String,

    /// Version to set as default. If not specified, uses the latest version.
    #[arg()]
    pub version: Option<String>,
}

// Common arguments for repository operations
#[derive(Parser, Clone)]
pub struct CmdArgs {
    /// GitHub user and repository in the format USERNAME/REPO
    /// e.g. pirafrank/rust_exif_renamer
    #[arg(required = true, value_parser = validate_repo_format)]
    pub repo: String,

    /// Optional release tag (defaults to 'latest')
    #[arg(long, short)]
    pub tag: Option<String>,
}

// Specific structure for the update command
#[derive(Parser, Clone)]
pub struct UpdateArgs {
    /// Github slug in the format USERNAME/REPO
    #[arg(value_parser = validate_repo_format, required_unless_present_any = ["all"])]
    pub repo: Option<String>,

    /// Update all installed binaries
    #[arg(long, conflicts_with_all = ["repo"])]
    pub all: bool,
}

fn parse_shell(s: &str) -> Result<SupportedShell, String> {
    s.parse::<SupportedShell>().map_err(|e| {
        format!(
            "{}. Possible values: {}",
            e,
            SupportedShell::possible_values().join(", ")
        )
    })
}

// Structure for the completions command
#[derive(Parser, Clone)]
pub struct ShellIntegrationArgs {
    /// Shell type to generate completions for, integrate via init command, and more.
    /// Possible values: bash, elvish, fish, nushell (or nu), powershell (or pwsh), xonsh, zsh
    #[arg(long, short, value_parser = parse_shell)]
    pub shell: SupportedShell,
}

// Structure for the unlink command
#[derive(Parser, Clone)]
pub struct UnlinkArgs {
    /// Name of the binary to unlink from the bin directory
    #[arg(required = true, value_parser = validate_binary_name)]
    pub binary_name: String,

    /// Skip confirmation prompt
    #[arg(short, long)]
    pub yes: bool,
}

// Structure for the list command
#[derive(Parser, Clone)]
pub struct ListArgs {
    /// GitHub user and repository in the format USERNAME/REPO
    /// e.g. pirafrank/rust_exif_renamer
    #[arg(required = false, value_parser = validate_repo_format)]
    pub repo: Option<String>,
}

// Structure for the which command
#[derive(Parser, Clone)]
pub struct WhichArgs {
    /// Name of the binary to look up
    #[arg(required = true, value_parser = validate_binary_name)]
    pub binary_name: String,
}

// Structure for the what command
#[derive(Parser, Clone)]
pub struct WhatArgs {
    /// GitHub user and repository in the format USERNAME/REPO
    /// e.g. pirafrank/rust_exif_renamer
    #[arg(required = true, value_parser = validate_repo_format)]
    pub repo: String,
}

// Structure for the uninstall command
#[derive(Parser, Clone)]
#[command(group(ArgGroup::new("what_to_uninstall").required(true).args(["version", "all"])))]
pub struct UninstallArgs {
    /// GitHub user and repository in the format USERNAME/REPO
    #[arg(required = true, value_parser = validate_repo_format)]
    pub repo: String,

    /// Version to uninstall
    #[arg(long, short = 'v', group = "what_to_uninstall")]
    pub version: Option<String>,

    /// Uninstall all versions of the slug
    #[arg(long, group = "what_to_uninstall")]
    pub all: bool,

    /// Skip confirmation prompt
    #[arg(short, long)]
    pub yes: bool,
}

// Command line interface
#[derive(Subcommand, Clone)]
pub enum Cmd {
    /// Only perform download for the platform in current directory. Do not install.
    Download(CmdArgs),

    /// Download binary for the platform and install it
    Install(CmdArgs),

    /// List all installed binaries and their versions
    List(ListArgs),

    /// Show which repository provides a binary
    Which(WhichArgs),

    /// List all binaries provided by the latest version of a repository
    What(WhatArgs),

    /// Set an installed version of a slug as the default one
    Use(UseArgs),

    /// Update installed binaries of a slug or all installed binaries to their latest versions
    Update(UpdateArgs),

    /// Remove binary from PATH. Use 'poof use' to re-add it
    Unlink(UnlinkArgs),

    /// Uninstall a version or all versions of a repository
    Uninstall(UninstallArgs),

    /// Persistently add poof's bin directory to your shell PATH
    Enable(ShellIntegrationArgs),

    /// Check if poof's bin directory is in the PATH
    Check,

    /// Generate shell completions to stdout
    Completions(ShellIntegrationArgs),

    /// Generate shell-specific init script to add poof bin directory to PATH
    Init(ShellIntegrationArgs),

    /// Empty the cache directory
    Clean,

    /// Show install and environment information
    Info,

    /// Show version information
    Version,
}

#[derive(Parser)]
#[command(
  name = APP_NAME,
  author = AUTHOR,
version = VERSION,
  about = short_description(),
  long_version = long_version(),
  help_template = "\n\n{name} - {about}\n\n\
    {usage-heading} {usage}\n\n\
    {all-args}{after-help}",
  after_help = format!("For more information, visit: {}\n\n\
    If you encounter any issues, please report them at:\n{}/issues\n",
    THIS_REPO_URL, THIS_REPO_URL),
)]
pub struct Cli {
    /// Command to execute
    #[command(subcommand)]
    pub command: Cmd,
}
