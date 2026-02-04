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
mod output;
mod utils;

// Use modules locally
use crate::cli::{Cli, Cmd};
use crate::constants::THIS_REPO_URL;
use crate::models::slug::Slug;
use crate::models::spell::Spell;
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
    // Set up logging using RUST_LOG environment variable (defaults to info level)
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .format_module_path(false)
        .format_target(false)
        .format(|buf, record| {
            use log::Level;
            use std::io::Write;

            // info!() shows just the message, others show colored level prefix
            match record.level() {
                Level::Info => writeln!(buf, "{}", record.args()),
                _ => {
                    let level_style = buf.default_level_style(record.level());
                    write!(buf, "{}", level_style.render())?;
                    write!(buf, "[{}]", record.level())?;
                    write!(buf, "{}", level_style.render_reset())?;
                    writeln!(buf, " {}", record.args())
                }
            }
        })
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
            if let Some(ref version) = args.version {
                info!(
                    "Setting version '{}' as default for {}",
                    version, &args.repo
                );
            } else {
                info!(
                    "Setting the newest installed version as default for {}",
                    &args.repo
                );
            }
            if let Err(e) = commands::make_default::set_default(&args.repo, args.version.as_deref())
            {
                error!("Cannot set default version: {}", e);
                std::process::exit(110);
            }
        }
        Cmd::List(args) => {
            let list: Vec<Spell>;
            if let Some(ref repo) = args.repo {
                let repo = Slug::new(repo)?;
                list = vec![commands::list::list_installed_versions_per_slug(&repo)?];
            } else {
                list = commands::list::list_installed_spells();
            }

            // output the list
            if list.is_empty() {
                info!("No installed binaries found.");
            } else {
                output!("");
                output!("{:<40}\t{}", "Repository", "Versions");
                output!("{:<40}\t{}", "----------", "--------");
                for asset in list {
                    output!(
                        "{:<40}\t{}",
                        asset.get_name(),
                        asset.get_versions().to_string_vec().join(", ")
                    );
                }
            }
        }
        Cmd::Which(args) => {
            commands::which::run_which(args)?;
        }
        Cmd::What(args) => {
            commands::what::run_what(args)?;
        }
        Cmd::Update(args) => {
            commands::update::process_update(args)?; // we use ? here, it returns a Result
        }
        Cmd::Check => {
            commands::check::check_if_bin_in_path();
        }
        Cmd::Version => {
            output!("{}", crate::core::platform_info::long_version());
        }
        Cmd::Info => {
            commands::info::show_info();
        }
        Cmd::Enable(args) => {
            commands::enable::run(args.shell);
        }
        Cmd::Clean => {
            commands::clean::run_clean()?;
        }
        Cmd::Unlink(args) => {
            commands::unlink::run_unlink(args)?;
        }
        Cmd::Uninstall(args) => {
            commands::uninstall::run_uninstall(args)?;
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

fn main() {
    if let Err(e) = run() {
        if log::log_enabled!(log::Level::Debug) {
            // Show full chain in debug mode
            error!("{:?}", e);
        } else {
            // Show only top-level error in normal mode
            error!("{}", e);
        }
        std::process::exit(1);
    }
}
