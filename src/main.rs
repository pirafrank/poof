use std::io::Write;

use anyhow::{bail, Context, Result};
use clap::Parser;
use log::{debug, error, info};

// Declare modules
mod cli;
mod commands;
mod constants;
mod core;
mod files;
mod github;
mod models;
mod utils;

// Use modules locally
use crate::cli::{Cli, Cmd};
use crate::constants::THIS_REPO_URL;
use crate::utils::semver::SemverStringConversion;

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
                std::env::current_dir().context("Cannot determine current directory")?;
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
                        "Cannot download asset for {} version {}",
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
                error!("Cannot set default version: {}", e);
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
        Cmd::Enable(args) => {
            commands::enable::run(args.shell);
        }
        Cmd::Clean => {
            commands::clean::run_clean()?;
        }
        Cmd::Completions(args) => {
            commands::completions::generate_completions(args.shell);
        }
        Cmd::Init(args) => {
            commands::init::generate_init_script(args.shell);
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
