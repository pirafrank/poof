use reqwest;
use serde::Deserialize;

mod archives;
mod filesys;

#[derive(Deserialize, Debug)]
struct ReleaseAsset {
    name: String,
    content_type: String,
    //size: u64,
    browser_download_url: String,
    // Add other fields if needed
}

#[derive(Deserialize, Debug)]
struct Release {
    tag_name: String,
    name: String,
    published_at: String, // Consider using chrono::DateTime<chrono::Utc> for proper date handling
    assets: Vec<ReleaseAsset>,
    // Add other fields if needed
}

fn main() {
    if !is_linux() {
        println!("Not running on Linux");
        std::process::exit(100);
    }

    println!("Running on Linux");
    let config_dir = filesys::get_config_dir();
    println!(
        "Config directory: {}",
        config_dir.ok_or(libc::ENOENT).unwrap()
    );
    let data_dir: String = filesys::get_data_dir().ok_or(libc::ENOENT).unwrap();
    println!("Data directory: {}", data_dir);

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <username>/<repo>", args[0]);
        std::process::exit(1);
    }
    let repo = &args[1];

    let assets: Vec<ReleaseAsset> = get_list_of_chances(repo);
    let binaries: Vec<ReleaseAsset> = assets.into_iter()
        .filter(|asset| poof::is_env_compatible(&asset.name))
        .collect();

    if binaries.is_empty() {
        println!("No compatible pre-built binaries found.");
        std::process::exit(100);
    }
    println!("Compatible binaries found:");
    for binary in &binaries {
        println!("{}", binary.name);
    }

    println!("Downloading {}...", binaries[0].name);
    let binary = &binaries[0];
    let binary_url = &binary.browser_download_url;
    println!("Downloading: {}", binary_url);
    let response = reqwest::blocking::get(binary_url).unwrap();
    if response.status().is_success() {
        // Convert repo path to filesystem-friendly format by replacing '/' with OS separator
        let repo_path = repo.replace('/', &std::path::MAIN_SEPARATOR.to_string());
        // Create path for the download: data_dir/repo_path
        let download_dir = std::path::Path::new(&data_dir).join(&repo_path);
        // Ensure the directory exists
        std::fs::create_dir_all(&download_dir).unwrap();

        // Create the file path and open it for writing
        let archive_path = download_dir.join(&binary.name);
        let mut file = std::fs::File::create(&archive_path).unwrap();

        println!("Saving to: {}", archive_path.display());
        std::io::copy(&mut response.bytes().unwrap().as_ref(), &mut file).unwrap();
        println!("Download complete.");

        archives::extract_to_dir_depending_on_content_type(
            &binary.content_type,
            &archive_path,
            &data_dir,
        )
        .expect("Failed to extract archive");
        println!("Extracted to: {}", data_dir);

        // println!("Making {} executable", file_path.display());
        // #[cfg(target_os = "linux")]
        // {
        //     use std::os::unix::fs::PermissionsExt;
        //     let mut perms = file.metadata().unwrap().permissions();
        //     perms.set_mode(0o755);
        //     std::fs::set_permissions(&file_path, perms).unwrap();
        // }
    } else {
        println!("Failed to download: {}", binary.name);
    }

    println!("Done.");
    std::process::exit(0);
}

fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

// TODO: make async, support multithreaded execution
fn get_list_of_chances(repo: &String) -> Vec<ReleaseAsset> {
    let release_url = get_release_url(&repo, None);
    println!("Release URL: {}", release_url);
    let client = reqwest::blocking::Client::new();

    // Make the request
    match client
        .get(&release_url)
        .header("User-Agent", "pirafrank/poof") // Keep User-Agent header for GitHub API
        .header("Accept", "application/vnd.github.v3+json")
        .send()
    {
        Ok(response) => {
            println!("Response Status: {}", response.status());
            if response.status().is_success() {
                // Attempt to parse the JSON response into a Vec<Release>
                match response.json::<Release>() {
                    Ok(release) => {
                        println!("Latest release tag: {}", release.tag_name);
                        println!("Latest release name: {}", release.name);
                        println!("Published at: {}", release.published_at);
                        println!("Available assets:");
                        for asset in &release.assets {
                            println!("{}", asset.name);
                        }
                        return release.assets;
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
            std::process::exit(99);
        }
    }
}

fn get_release_url(repo: &String, tag: Option<&String>) -> String {
    match tag {
        Some(tag) => format!(
            "https://api.github.com/repos/{}/releases/tags/{}",
            repo, tag
        ),
        None => format!(
            "https://api.github.com/repos/{}/releases/latest",
            repo
        ),
    }
}
