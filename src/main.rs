use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ReleaseAsset {
    name: String,
    //content_type: String,
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
    let config_dir = get_config_dir();
    println!(
        "Config directory: {}",
        config_dir.ok_or(libc::ENOENT).unwrap()
    );
    let data_dir = get_data_dir();
    println!("Data directory: {}", data_dir.ok_or(libc::ENOENT).unwrap());

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <username>/<repo>", args[0]);
        std::process::exit(1);
    }
    let repo = &args[1];

    let assets: Vec<ReleaseAsset> = get_list_of_chances(repo);
    let binaries: Vec<ReleaseAsset> = assets.into_iter()
        .filter(|asset| foop::is_env_compatible(&asset.name))
        .collect();

    if binaries.is_empty() {
        println!("No compatible binaries found.");
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
        let mut file = std::fs::File::create(&binary.name).unwrap();
        std::io::copy(&mut response.bytes().unwrap().as_ref(), &mut file).unwrap();
        println!("Downloaded: {}", binary.name);
    } else {
        println!("Failed to download: {}", binary.name);
    }

    println!("All downloads completed.");
    println!("Done.");
    std::process::exit(0);
}

// TODO: make async, support multithreaded execution
fn get_list_of_chances(repo: &String) -> Vec<ReleaseAsset> {
    let release_url = get_release_url(&repo, None);
    println!("Release URL: {}", release_url);
    let client = reqwest::blocking::Client::new();

    // Make the request
    match client
        .get(&release_url)
        .header("User-Agent", "rust_exif_renamer") // Keep User-Agent header for GitHub API
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

fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

// ~/.config/APPNAME/config.json
fn get_config_dir() -> Option<String> {
    if let Some(app_name) = option_env!("CARGO_PKG_NAME") {
        let home_dir = dirs::home_dir()?;
        let config_dir = home_dir.join(".config").join(app_name);
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).ok()?;
        }
        Some(config_dir.display().to_string())
    } else {
        None
    }
}

// ~/.local/share/APPNAME
fn get_data_dir() -> Option<String> {
    if let Some(app_name) = option_env!("CARGO_PKG_NAME") {
        let home_dir = dirs::home_dir()?;
        let data_dir = home_dir.join(".local").join("share").join(app_name);
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).ok()?;
        }
        Some(data_dir.display().to_string())
    } else {
        None
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
