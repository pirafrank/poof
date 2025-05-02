use std::io::Write;

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use lazy_static::lazy_static;
use log::{debug, error, info};
use regex::Regex;

mod archives;
mod commands;
mod constants;
mod datadirs;
mod filesys;
mod github;
mod models;
mod platform_info;
mod selector;
mod semver_utils;
mod utils;

use crate::constants::*;
use crate::selector::is_env_compatible;
use github::client::{get_asset, get_release};
use semver_utils::SemverStringConversion;

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

// Common arguments for repository operations
#[derive(Parser, Clone)]
struct CmdArgs {
    /// GitHub user and repository in the format USERNAME/REPO
    /// e.g. pirafrank/rust_exif_renamer
    #[arg(required = true, value_parser = validate_repo_format)]
    repo: String,

    /// Optional release tag (defaults to 'latest')
    #[arg(long, short)]
    tag: Option<String>,
}

// Command line interface
#[derive(Subcommand, Clone)]
enum Cmd {
    /// Only download binary for the platform in current directory. No install.
    Download(CmdArgs),

    /// Download binary for the platform and install it
    Install(CmdArgs),

    /// List installed binaries and their versions
    List,

    /// Make an installed version the one to be used by default
    Use(CmdArgs),

    /// Persistently add poof’s bin directory to your shell PATH
    Enable,

    /// Check if poof's bin directory is in the PATH
    Check,

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
  version = platform_info::long_version(),
  about = platform_info::short_description(),
  long_version = platform_info::long_version(),
  help_template = "\n\n{name} - {about}\n\n\
    {usage-heading} {usage}\n\n\
    {all-args}{after-help}",
  after_help = format!("For more information, visit: {}\n\n\
    If you encounter any issues, please report them at:\n{}/issues\n",
    THIS_REPO_URL, THIS_REPO_URL),
)]
struct Cli {
    /// Command to execute
    #[command(subcommand)]
    command: Cmd,

    /// Enable debug logging
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>, // default to INFO
}

fn is_supported_os() -> bool {
    cfg!(any(target_os = "linux", target_os = "macos"))
}

fn main() {
    if !is_supported_os() {
        error!("Sorry, {} is currenly unsupported.", std::env::consts::OS);
        error!(
            "Please open an issue at {}/issues, to ask for support.",
            THIS_REPO_URL
        );
        std::process::exit(100);
    }

    // Parse command-line arguments
    let cli = Cli::parse();
    // Set up logging
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .init();

    // Execute different logic based on command
    match &cli.command {
        Cmd::Download(args) => {
            info!(
                "Downloading {} {} to current dir",
                &args.repo,
                args.tag.as_deref().unwrap_or("(latest)")
            );
            let current_dir =
                std::env::current_dir().expect("Failed to determine current directory");
            debug!("Working directory: {}", current_dir.display());

            let release = get_release(&args.repo, args.tag.as_deref());
            let binary = get_asset(&release, is_env_compatible);
            commands::download::download_binary(
                binary.name(),
                binary.browser_download_url(),
                &current_dir,
            );
        }
        Cmd::Install(args) => {
            info!(
                "Installing {} {}",
                &args.repo,
                args.tag.as_deref().unwrap_or("(latest)")
            );
            commands::install::process_install(&args.repo, args.tag.as_deref());
        }
        Cmd::Use(args) => {
            let version = args.tag.as_deref().unwrap_or("latest");
            info!(
                "Setting version '{}' as default for {}",
                version, &args.repo
            );
            if let Err(e) = commands::make_default::set_default(&args.repo, version) {
                error!("Failed to set default version: {}", e);
                std::process::exit(110);
            }
            info!("Version '{}' set as default.", version);
        }
        Cmd::List => {
            let list = commands::list::list_installed_assets();
            if list.is_empty() {
                info!("No installed binaries found.");
            } else {
                let mut stdout = std::io::stdout().lock();
                writeln!(stdout).unwrap();
                writeln!(stdout, "{:<40} {:<15}", "Repository", "Versions").unwrap();
                writeln!(stdout, "{:<40} {:<15}", "----------", "--------").unwrap();
                for asset in list {
                    writeln!(
                        stdout,
                        "{:<40} {:?}",
                        asset.get_name(),
                        asset.get_versions().to_string_vec()
                    )
                    .unwrap();
                }
                writeln!(stdout).unwrap();
                drop(stdout); // explicitly release the lock
            }
        }
        Cmd::Check => {
            commands::check::check_if_bin_in_path();
        }
        Cmd::Version => {
            println!("{}", platform_info::long_version());
        }
        Cmd::Info => {
            commands::info::show_info();
        }
        Cmd::Debug => {
            commands::info::show_info();
        }
        Cmd::Enable => {
            commands::enable::run();
        }
    }
}
