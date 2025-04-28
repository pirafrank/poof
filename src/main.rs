use std::path::PathBuf;
use std::{fs::File, path::Path};

use clap::{Parser, Subcommand};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, error, info, warn};
use serde::Deserialize;

mod archives;
mod filesys;
mod platform_info;
mod utils;

// Constants
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const THIS_REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");
const GITHUB_API_URL: &str = "https://api.github.com/repos";
const GITHUB_API_USER_AGENT: &str = "pirafrank/poof";
const GITHUB_API_ACCEPT: &str = "application/vnd.github.v3+json";

// Common arguments for repository operations
#[derive(Parser, Clone)]
struct CmdArgs {
    /// GitHub user and repository in the format USERNAME/REPO
    /// e.g. pirafrank/rust_exif_renamer
    #[arg(required = true)]
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

    /// Check if poof's bin directory is in the PATH
    Check,

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

#[derive(Clone, Deserialize, Debug)]
struct ReleaseAsset {
    name: String,
    content_type: String,
    //size: u64,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct Release {
    tag_name: String,
    published_at: String, // Consider using chrono::DateTime<chrono::Utc> for proper date handling
    assets: Vec<ReleaseAsset>,
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
            let binary = get_asset(&release);
            download_binary(&binary.name, &binary.browser_download_url, &current_dir);
        }
        Cmd::Install(args) => {
            info!(
                "Installing {} {}",
                &args.repo,
                args.tag.as_deref().unwrap_or("(latest)")
            );
            process_install(&args.repo, args.tag.as_deref());
        }
        Cmd::Check => {
            check_if_bin_in_path();
        }
        Cmd::Version => {
            println!("{}", platform_info::long_version());
        }
        Cmd::Debug => {
            platform_info::debug_info();
        }
    }
}

fn process_install(repo: &str, tag: Option<&str>) {
    // let config_dir = filesys::get_config_dir().ok_or(libc::ENOENT).unwrap();
    // info!("Config directory: {}", config_dir);
    let cache_dir: PathBuf = filesys::get_cache_dir().ok_or(libc::ENOENT).unwrap();
    debug!("Cache directory: {}", cache_dir.display());

    // download binary
    let release = get_release(repo, tag);
    let binary = get_asset(&release);
    let download_to = get_install_path(&cache_dir, repo, &release.tag_name);
    download_binary(&binary.name, &binary.browser_download_url, &download_to);

    // extract binary
    let archive_path = download_to.join(&binary.name);
    archives::extract_to_dir_depending_on_content_type(
        &binary.content_type,
        &archive_path,
        &download_to,
    )
    .expect("Failed to extract archive");
    debug!("Extracted to: {}", download_to.display());

    // install binary
    install_binary(&archive_path, repo, &release.tag_name);
    info!("{} installed successfully.", binary.name);
    check_if_bin_in_path();
    std::process::exit(0);
}

fn install_binary(archive_path: &Path, repo: &str, version: &str) {
    let data_dir: PathBuf = filesys::get_data_dir().ok_or(libc::ENOENT).unwrap();
    debug!("Data directory: {}", data_dir.display());
    let install_dir = get_install_path(&data_dir, repo, version);
    debug!("Installing to: {}", install_dir.display());
    // Create the installation directory if it doesn't exist
    if !install_dir.exists() {
        std::fs::create_dir_all(&install_dir).unwrap();
    } else {
        warn!(
            "Version is already installed. Check content in {} dir.",
            install_dir.display()
        );
        warn!("If you want to reinstall, please remove the directory first.");
        std::process::exit(0);
    }

    let execs_to_install: Vec<PathBuf> =
        filesys::find_exec_files_from_extracted_archive(archive_path);
    for exec in execs_to_install {
        let file_name = exec.file_name().unwrap();
        let installed_exec = install_dir.join(file_name);
        debug!("Copying {} to {}", exec.display(), installed_exec.display());
        if let Err(e) = std::fs::copy(&exec, &installed_exec) {
            error!(
                "Error copying {} to {}: {}",
                exec.display(),
                installed_exec.display(),
                e
            );
            error!("Installation failed.");
            std::process::exit(103);
        }
        debug!("Making {} executable", file_name.to_string_lossy());
        // Set executable permissions, platform-specific
        // Note: Windows does not require setting executable permissions
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            // Unix-like systems require setting executable permissions
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&installed_exec).unwrap().permissions();
            // Add executable bits to current permissions (equivalent to chmod +x)
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&installed_exec, perms).unwrap();
            debug!(
                "Set executable permissions for {}",
                installed_exec.display()
            );
            // Create a symlink in the bin directory
            let bin_dir: PathBuf = filesys::get_bin_dir().ok_or(libc::ENOENT).unwrap();
            let symlink_path = bin_dir.join(file_name);
            debug!(
                "Creating symlink {} -> {}",
                symlink_path.display(),
                installed_exec.display()
            );
            if let Err(e) = filesys::symlink(&installed_exec, &symlink_path) {
                error!(
                    "Cannot symlink {} -> {}.\n\nInstallation failed. {}",
                    symlink_path.display(),
                    installed_exec.display(),
                    e
                );
            } else {
                info!(
                    "Symlink created: {} -> {}",
                    symlink_path.display(),
                    installed_exec.display()
                );
            }
        }
    }
}

// Function to handle downloading and potentially installing binaries
fn download_binary(filename: &String, download_url: &String, download_to: &PathBuf) {
    info!("Downloading {} from {}", filename, download_url);
    let response = reqwest::blocking::get(download_url).unwrap();
    if response.status().is_success() {
        // Ensure the directory exists
        std::fs::create_dir_all(download_to).unwrap();

        // Create the file path and open it for writing
        let archive_path = download_to.join(filename);
        let mut file = File::create(&archive_path).unwrap();

        debug!("Saving to: {}", archive_path.display());
        std::io::copy(&mut response.bytes().unwrap().as_ref(), &mut file).unwrap();
        info!("Download complete.");
    } else {
        error!("Download failed!");
        std::process::exit(99)
    }
}

fn get_install_path(base: &Path, repo: &str, version: &str) -> PathBuf {
    // Convert repo path to filesystem-friendly format by replacing '/' with OS separator
    let repo_path = repo.replace('/', std::path::MAIN_SEPARATOR_STR);
    // Creating path as: base_dir/username/reponame/version
    base.join(&repo_path).join(version)
}

fn get_asset(release: &Release) -> ReleaseAsset {
    let binaries: Vec<ReleaseAsset> = release
        .assets
        .iter()
        .filter(|asset| poof::is_env_compatible(&asset.name))
        .cloned()
        .collect();

    if binaries.is_empty() {
        error!("No compatible pre-built binaries found.");
        std::process::exit(100);
    }
    debug!("Compatible binaries found:");
    for binary in &binaries {
        debug!("\t{}", binary.name);
    }
    if binaries.len() > 1 {
        warn!("Multiple compatible binaries found. Downloading first...");
        // TODO: allow to specify which binary to download via explicit URL given to 'install' command
    }
    // Return the first compatible binary
    binaries[0].clone()
}

fn get_release(repo: &str, tag: Option<&str>) -> Release {
    let release_url = get_release_url(repo, tag);
    info!("Release URL: {}", release_url);
    let client = reqwest::blocking::Client::new();

    // Make the request
    match client
        .get(&release_url)
        .header("User-Agent", GITHUB_API_USER_AGENT) // Keep User-Agent header for GitHub API
        .header("Accept", GITHUB_API_ACCEPT)
        .send()
    {
        Ok(response) => {
            debug!("Response Status: {}", response.status());
            if response.status().is_success() {
                // Attempt to parse the JSON response into a Vec<Release>
                match response.json::<Release>() {
                    Ok(release) => {
                        if tag.is_some() {
                            info!("Selected release tag: {}", tag.unwrap());
                        } else {
                            info!("Current latest release tag: {}", release.tag_name);
                        }
                        info!("Published at: {}", release.published_at);
                        debug!("Available assets:");
                        for asset in &release.assets {
                            debug!("\t{}", asset.name);
                        }
                        release
                    }
                    Err(e) => {
                        error!("Failed to parse JSON response: {}", e);
                        std::process::exit(101);
                    }
                }
            } else {
                error!("Request failed with status: {}", response.status());
                std::process::exit(102);
            }
        }
        Err(e) => {
            error!("Failed to send request: {}", e);
            std::process::exit(91);
        }
    }
}

fn get_release_url(repo: &str, tag: Option<&str>) -> String {
    match tag {
        Some(tag) => format!("{}/{}/releases/tags/{}", GITHUB_API_URL, repo, tag),
        None => format!("{}/{}/releases/latest", GITHUB_API_URL, repo),
    }
}

fn check_if_bin_in_path() {
    let bin_dir: PathBuf = filesys::get_bin_dir().ok_or(libc::ENOENT).unwrap();
    let position = platform_info::check_dir_in_path(bin_dir.to_str().unwrap());
    match position {
        0 => {
            warn!("Bin directory not found in PATH.");
            warn!(
                "Please add {} to your PATH. For example, run: \n\n{}\n",
                bin_dir.display(),
                get_export_command()
            );
            warn!("This is required to run the installed binaries.");
        }
        1 => info!("It looks good. Bin directory is the first in PATH."),
        _ => {
            warn!("Bin directory is not the first in PATH.");
            warn!(
                "Please move {} to the beginning of your PATH.",
                bin_dir.display()
            );
        }
    }
}

fn get_export_command() -> String {
    let bin_dir: PathBuf = filesys::get_bin_dir().ok_or(libc::ENOENT).unwrap();
    format!("export PATH=\"{}:$PATH\"", bin_dir.to_str().unwrap())
}
