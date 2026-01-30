use crate::constants::*;
use crate::core::platform_info::{long_version, short_description};
use crate::models::supported_shells::SupportedShell;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use lazy_static::lazy_static;
use regex::Regex;

// Constants

lazy_static! {
    static ref REPO_REGEX: Regex = Regex::new(r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$").unwrap();
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

#[derive(Parser, Clone)]
pub struct UseArgs {
    /// GitHub user and repository in the format USERNAME/REPO
    /// e.g. pirafrank/rust_exif_renamer
    #[arg(required = true, value_parser = validate_repo_format)]
    pub repo: String,

    /// version to set as default
    #[arg(required = true)]
    pub version: String,
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
    #[arg(value_parser = validate_repo_format, required_unless_present_any = ["all", "update_self"])]
    pub repo: Option<String>,

    /// Update all installed binaries
    #[arg(long, conflicts_with_all = ["repo", "update_self"])]
    pub all: bool,

    /// Update poof itself to the latest version.
    /// It works only if the binary has not been installed from a package manager.
    /// If you installed poof from a package manager, update via that package manager instead.
    #[arg(long = "self", conflicts_with_all = ["repo", "all"])]
    pub update_self: bool,
}

fn parse_shell(s: &str) -> Result<SupportedShell, String> {
    SupportedShell::from_str(s).ok_or_else(|| {
        format!(
            "unsupported shell: '{}'. Possible values: {}",
            s,
            SupportedShell::possible_values().join(", ")
        )
    })
}

// Structure for the completions command
#[derive(Parser, Clone)]
pub struct CompletionsArgs {
    /// Shell type to generate completions for.
    /// Possible values: bash, elvish, fish, nushell (or nu), powershell (or pwsh), xonsh, zsh
    #[arg(long, short, value_parser = parse_shell)]
    pub shell: SupportedShell,
}

// Structure for the init command
#[derive(Parser, Clone)]
pub struct InitArgs {
    /// Shell type to generate init script for.
    /// Possible values: bash, elvish, fish, nushell (or nu), powershell (or pwsh), xonsh, zsh
    #[arg(long, short, value_parser = parse_shell)]
    pub shell: SupportedShell,
}

// Structure for the enable command
#[derive(Parser, Clone)]
pub struct EnableArgs {
    /// Shell type to configure.
    /// Possible values: bash, elvish, fish, nushell (or nu), powershell (or pwsh), xonsh, zsh
    #[arg(long, short, value_parser = parse_shell)]
    pub shell: SupportedShell,
}

// Command line interface
#[derive(Subcommand, Clone)]
pub enum Cmd {
    /// Only download binary for the platform in current directory. Do not perform installation.
    Download(CmdArgs),

    /// Download binary for the platform and install it
    Install(CmdArgs),

    /// List all installed binaries and their versions
    List,

    /// Set an installed version of a slug as the default one
    Use(UseArgs),

    /// Update installed binaries of a slug or all installed binaries to their latest versions
    Update(UpdateArgs),

    /// Persistently add poof's bin directory to your shell PATH
    Enable(EnableArgs),

    /// Check if poof's bin directory is in the PATH
    Check,

    /// Generate shell completions to stdout
    Completions(CompletionsArgs),

    /// Generate shell-specific init script to add poof bin directory to PATH
    Init(InitArgs),

    /// Empty the cache directory
    Clean,

    /// Show install and environment information
    Info,

    /// Show version information
    Version,

    /// Show debug information
    #[command(hide = true)]
    Debug,
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

    /// Enable debug logging
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>, // default to INFO
}
