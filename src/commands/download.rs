//! Main file handling 'download' command

use std::{fs::File, path::PathBuf};

use log::{debug, error, info};

// Function to handle downloading and potentially installing binaries
pub fn download_binary(filename: &String, download_url: &String, download_to: &PathBuf) {
    info!("Downloading {}\nfrom {}", filename, download_url);
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
