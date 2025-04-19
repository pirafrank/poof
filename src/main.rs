use std::fs::File;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use reqwest;
use serde::Deserialize;

mod archives;
mod filesys;

// Version constants
const VERSION: &str = env!("CARGO_PKG_VERSION");
const COMMIT: &str = env!("GIT_COMMIT_HASH");
const BUILD_DATE: &str = env!("BUILD_DATE");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const THIS_REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");
const GITHUB_API_URL: &str = "https://api.github.com/repos";
const GITHUB_API_USER_AGENT: &str = "pirafrank/poof";
const GITHUB_API_ACCEPT: &str = "application/vnd.github.v3+json";

/// Returns a static string containing the version information.
/// It uses Box::leak to convert a String into a &'static str.
/// This is a workaround to avoid using a global static variable.
fn long_version() -> &'static str {
    Box::leak(
        format!(
            "Version: {}\nCommit: {}\nBuild Date: {}",
            VERSION, COMMIT, BUILD_DATE
        )
        .into_boxed_str(),
    )
}

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
    Get(CmdArgs),

    /// Download binary for the platform and install it
    Install(CmdArgs),

    /// Show version information
    Version,
}

#[derive(Parser)]
#[command(
  author = AUTHOR,
  version = long_version(),
  about = DESCRIPTION,
  long_version = long_version()
)]
struct Cli {
    /// Command to execute
    #[command(subcommand)]
    command: Cmd,
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

fn is_supported_arch() -> bool {
    cfg!(target_os = "linux")
}

fn main() {
    if !is_supported_arch() {
        println!("Sorry, {} is currenly unsupported.", std::env::consts::OS);
        println!("Please open an issue at {}/issues, to ask for support.", THIS_REPO_URL);
        std::process::exit(100);
    }

    // Parse command-line arguments
    let cli = Cli::parse();

    // Execute different logic based on command
    match &cli.command {
        Cmd::Get(args) => {
            println!("Executing GET for repository: {}", &args.repo);
            let current_dir = std::env::current_dir()
                .expect("Failed to determine current directory");
            println!("Working directory: {}", current_dir.display());

            let release = get_release(&args.repo, args.tag.as_ref().map(String::as_str));
            let binary = get_asset(&release);
            download_binary(&binary.name, &binary.browser_download_url, &current_dir);
        }
        Cmd::Install(args) => {
            println!("Executing INSTALL for repository: {}", &args.repo);
            process_install(&args.repo, args.tag.as_ref().map(String::as_str));
        }
        Cmd::Version => {
            println!("{}", long_version());
        }
    }
}

fn process_install(repo: &str, tag: Option<&str>) {
    // let config_dir = filesys::get_config_dir().ok_or(libc::ENOENT).unwrap();
    // println!("Config directory: {}", config_dir);
    let cache_dir: PathBuf  = filesys::get_cache_dir().ok_or(libc::ENOENT).unwrap();
    println!("Cache directory: {}", cache_dir.display());

    // download binary
    let release = get_release(repo, tag);
    let binary = get_asset(&release);
    let download_to = get_target_path(&cache_dir, &repo, &release.tag_name);
    download_binary(&binary.name, &binary.browser_download_url, &download_to);

    // extract binary
    let archive_path = download_to.join(&binary.name);
    archives::extract_to_dir_depending_on_content_type(
        &binary.content_type,
        &archive_path,
        &download_to,
    )
    .expect("Failed to extract archive");
    println!("Extracted to: {}", download_to.display());

    // install binary
    install_binary(&archive_path, repo, &release.tag_name);
    println!("{} installed successfully.", binary.name);
    std::process::exit(0);
}

fn install_binary(archive_path: &PathBuf, repo: &str, version: &str) {
    let data_dir: PathBuf = filesys::get_data_dir().ok_or(libc::ENOENT).unwrap();
    println!("Data directory: {}", data_dir.display());
    let install_dir = get_target_path(&data_dir, &repo, &version);
    println!("Installing to: {}", install_dir.display());
    // Create the installation directory if it doesn't exist
    if !install_dir.exists() {
        std::fs::create_dir_all(&install_dir).unwrap();
    }
    let bin_dir: PathBuf = filesys::get_bin_dir().ok_or(libc::ENOENT).unwrap();
    println!("Bin directory: {}", bin_dir.display());

    let execs_to_install: Vec<PathBuf> = filesys::find_exec_files_from_extracted_archive(archive_path);
    for exec in execs_to_install {
        let file_name = exec.file_name().unwrap();
        let installed_exec = install_dir.join(file_name);
        println!("Copying {} to {}", exec.display(), installed_exec.display());
        if let Err(e) = std::fs::copy(&exec, &installed_exec) {
            println!("Error copying {} to {}: {}", exec.display(), installed_exec.display(), e);
            println!("Installation failed.");
            std::process::exit(103);
        }
        println!("Making {} executable", file_name.to_string_lossy());
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
            println!("Set executable permissions for {}", installed_exec.display());
            // Create a symlink in the bin directory
            let symlink_path = bin_dir.join(file_name);
            println!("Creating symlink {} -> {}", symlink_path.display(), installed_exec.display());
            if let Err(e) = filesys::symlink(&installed_exec, &symlink_path) {
                println!("Error: Can't symlink. Installation failed. {}", e);
            } else {
                println!("Symlink created: {} -> {}", symlink_path.display(), installed_exec.display());
            }
        }
    }
}

// Function to handle downloading and potentially installing binaries
fn download_binary(
    filename: &String,
    download_url: &String,
    download_to: &PathBuf,
) {
    println!("Downloading {}\n\tfrom {}", filename, download_url);
    let response = reqwest::blocking::get(download_url).unwrap();
    if response.status().is_success() {
        // Ensure the directory exists
        std::fs::create_dir_all(&download_to).unwrap();

        // Create the file path and open it for writing
        let archive_path = download_to.join(filename);
        let mut file = File::create(&archive_path).unwrap();

        println!("Saving to: {}", archive_path.display());
        std::io::copy(&mut response.bytes().unwrap().as_ref(), &mut file).unwrap();
        println!("Download complete.");
    } else {
        println!("Error: Download failed!");
        std::process::exit(99)
    }
}

fn get_target_path(base: &PathBuf, repo: &str, version: &str) -> PathBuf {
    // Convert repo path to filesystem-friendly format by replacing '/' with OS separator
    let repo_path = repo.replace('/', &std::path::MAIN_SEPARATOR.to_string());
    // Creating path as: base_dir/username/reponame/version
    base.join(&repo_path).join(&version)
}

fn get_asset(release: &Release) -> ReleaseAsset {
    let binaries: Vec<ReleaseAsset> = release.assets
        .iter()
        .filter(|asset| poof::is_env_compatible(&asset.name))
        .cloned()
        .collect();

    if binaries.is_empty() {
        println!("No compatible pre-built binaries found.");
        std::process::exit(100);
    }
    println!("Compatible binaries found:");
    for binary in &binaries {
        println!("\t{}", binary.name);
    }
    if binaries.len() > 1 {
        println!("Multiple compatible binaries found. Downloading first...");
        // TODO: allow to specify which binary to download via explicit URL given to 'install' command
    }
    // Return the first compatible binary
    binaries[0].clone()
}

fn get_release(repo: &str, tag: Option<&str>) -> Release {
    let release_url = get_release_url(repo, tag);
    println!("Release URL: {}", release_url);
    let client = reqwest::blocking::Client::new();

    // Make the request
    match client
        .get(&release_url)
        .header("User-Agent", GITHUB_API_USER_AGENT) // Keep User-Agent header for GitHub API
        .header("Accept", GITHUB_API_ACCEPT)
        .send()
    {
        Ok(response) => {
            println!("Response Status: {}", response.status());
            if response.status().is_success() {
                // Attempt to parse the JSON response into a Vec<Release>
                match response.json::<Release>() {
                    Ok(release) => {
                        if tag.is_some() {
                            println!("Selected release tag: {}", tag.unwrap());
                        } else {
                            println!("Latest release tag: {}", release.tag_name);
                        }
                        println!("Published at: {}", release.published_at);
                        println!("Available assets:");
                        for asset in &release.assets {
                            println!("\t{}", asset.name);
                        }
                        return release;
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON response: {}", e);
                        std::process::exit(101);
                    }
                }
            } else {
                eprintln!("Request failed with status: {}", response.status());
                std::process::exit(102);
            }
        }
        Err(e) => {
            eprintln!("Failed to send request: {}", e);
            std::process::exit(91);
        }
    }
}

fn get_release_url(repo: &str, tag: Option<&str>) -> String {
    match tag {
        Some(tag) => format!(
            "{}/{}/releases/tags/{}",
            GITHUB_API_URL, repo, tag
        ),
        None => format!("{}/{}/releases/latest", GITHUB_API_URL, repo),
    }
}
