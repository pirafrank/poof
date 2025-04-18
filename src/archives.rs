use std::fs::File;
use std::path::PathBuf;
use zip::ZipArchive;
use tar::Archive;
use flate2::read::GzDecoder;
use xz2::read::XzDecoder;
use bzip2::read::BzDecoder;

/// Extracts an archive to a specified directory based on its content type.
/// Currently supports zip, tar.gz, tar.xz, and tar.bz2 formats.
///
/// # Arguments
/// * `content_type` - The content-type of the archive (e.g., "application/zip").
/// * `archive_path` - The name of the archive file.
/// * `target_dir` - The directory where the archive will be extracted.
///
/// # Returns
/// * `Ok(())` if the extraction was successful.
/// * `Err` if there was an error during extraction.
///
pub fn extract_to_dir_depending_on_content_type(content_type: &String,
    archive_path: &PathBuf,
    target_dir: &String,
) -> Result<(), Box<dyn std::error::Error>> {

    // Check the content type and extract accordingly
    match content_type.as_str() {
        "application/zip" => {
            println!("Extracting zip archive: {}", archive_path.display());
            let zip_file = File::open(archive_path)?;
            let mut archive = ZipArchive::new(zip_file)?;
            archive.extract(target_dir)?;
            println!("Successfully extracted zip archive to {}", target_dir);
        }
        "application/gzip" => { // Assuming this is tar.gz
            println!("Extracting tar.gz archive: {}", archive_path.display());
            let tar_gz_file = File::open(archive_path)?;
            let tar = GzDecoder::new(tar_gz_file);
            let mut archive = Archive::new(tar);
            archive.unpack(target_dir)?;
            println!("Successfully extracted tar.gz archive to {}", target_dir);
        }
        "application/x-xz" => { // Assuming this is tar.xz
            println!("Extracting tar.xz archive: {}", archive_path.display());
            let tar_xz_file = File::open(archive_path)?;
            let tar = XzDecoder::new(tar_xz_file);
            let mut archive = Archive::new(tar);
            archive.unpack(target_dir)?;
            println!("Successfully extracted tar.xz archive to {}", target_dir);
        }
        "application/x-bzip2" => { // Assuming this is tar.bz2
            println!("Extracting tar.bz2 archive: {}", archive_path.display());
            let tar_bz2_file = File::open(archive_path)?;
            let tar = BzDecoder::new(tar_bz2_file);
            let mut archive = Archive::new(tar);
            archive.unpack(target_dir)?;
            println!("Successfully extracted tar.bz2 archive to {}", target_dir);
        }
        // Add handling for plain tar if necessary
        // "application/x-tar" => { ... }
        _ => {
            // Consider returning an error instead of just printing
            // return Err(format!("Unsupported content type for extraction: {}", content_type).into());
            println!("Unsupported content type for extraction: {}", content_type);
            // If non-archive files should be ignored, remove the println!
        }
    }
    Ok(())
}

