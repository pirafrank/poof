use std::io::Write;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use lazy_static::lazy_static;
use log::{debug, error, info};
use regex::Regex;

mod commands;
mod constants;
mod core;
mod files;
mod github;
mod models;
mod utils;

use crate::constants::*;
use crate::core::platform_info::{long_version, short_description};
use utils::semver::SemverStringConversion;

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
struct UseArgs {
    /// GitHub user and repository in the format USERNAME/REPO
    /// e.g. pirafrank/rust_exif_renamer
    #[arg(required = true, value_parser = validate_repo_format)]
    repo: String,

    /// version to set as default
    #[arg(required = true)]
    version: String,
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

// Specific structure for the update command
#[derive(Parser, Clone)]
struct UpdateArgs {
    /// Github slug
    #[arg(value_parser = validate_repo_format, required_unless_present_any = ["all", "update_self"])]
    repo: Option<String>,

    /// Update all installed binaries
    #[arg(long, conflicts_with_all = ["repo", "update_self"])]
    all: bool,

    /// Update poof itself
    #[arg(long = "self", conflicts_with_all = ["repo", "all"])]
    update_self: bool,
}

// Structure for the completions command
#[derive(Parser, Clone)]
struct CompletionsArgs {
    /// Shell type to generate completions for
    #[arg(long, short, value_parser = parse_shell)]
    shell: commands::completions::SupportedShell,
}

fn parse_shell(s: &str) -> Result<commands::completions::SupportedShell, String> {
    commands::completions::SupportedShell::from_str(s).ok_or_else(|| {
        format!(
            "unsupported shell: '{}'. Possible values: {}",
            s,
            commands::completions::SupportedShell::possible_values().join(", ")
        )
    })
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
    Use(UseArgs),

    /// Update installed binaries to their latest versions
    Update(UpdateArgs),

    /// Persistently add poof's bin directory to your shell PATH
    Enable,

    /// Check if poof's bin directory is in the PATH
    Check,

    /// Empty cache directory
    Clean,

    /// Show install and environment information
    Info,

    /// Show version information
    Version,

    /// Show debug information
    #[command(hide = true)]
    Debug,

    /// Generate shell completions to stdout
    Completions(CompletionsArgs),
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

fn run() -> Result<()> {
    if !is_supported_os() {
        bail!(
            "Sorry, {} is currently unsupported. Please open an issue at {}/issues to ask for support.",
            std::env::consts::OS,
            THIS_REPO_URL
        );
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
                std::env::current_dir().context("Failed to determine current directory")?;
            debug!("Working directory: {}", current_dir.display());

            let (_, assets) = commands::install::select_assets(&args.repo, args.tag.as_deref())?;

            for asset in assets {
                commands::download::download_asset(
                    asset.name(),
                    asset.browser_download_url(),
                    &current_dir,
                )
                .with_context(|| {
                    format!(
                        "Failed to download asset for {} version {}",
                        args.repo,
                        args.tag.as_deref().unwrap_or("(latest)")
                    )
                })?;
            }
            info!("All done.");
        }
        Cmd::Install(args) => {
            info!(
                "Installing {} {}",
                &args.repo,
                args.tag.as_deref().unwrap_or("(latest)")
            );
            commands::install::install(&args.repo, args.tag.as_deref())?;
        }
        Cmd::Use(args) => {
            let version = &args.version;
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
            let list = commands::list::list_installed_spells();
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
        Cmd::Update(args) => {
            commands::update::process_update(args)?; // we use ? here, it returns a Result
        }
        Cmd::Check => {
            commands::check::check_if_bin_in_path();
        }
        Cmd::Version => {
            println!("{}", crate::core::platform_info::long_version());
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
        Cmd::Clean => {
            commands::clean::run_clean()?;
        }
        Cmd::Completions(args) => {
            commands::completions::generate_completions(args.shell);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    // call the main logic function
    let result = run();

    // log the error explicitly
    if let Err(e) = &result {
        error!("Execution failed: {:?}", e);
    }

    // return the result
    // if Ok(()) -> exit code 0
    // if Err(e) -> anyhow's Termination impl prints the error and exits with code 1
    result
}
