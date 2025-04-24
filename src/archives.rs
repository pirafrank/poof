use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use std::fs::File;
use std::path::PathBuf;
use tar::Archive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

/// Extracts an archive to a specified directory based on its content type.
/// Currently supports zip, tar.gz, tar.xz, and tar.bz2 formats.
///
/// # Arguments
/// * `content_type` - The content-type of the archive (e.g., "application/zip").
/// * `archive_path` - The name of the archive file.
/// * `extract_to` - The directory where the archive will be extracted.
///
/// # Returns
/// * `Ok(())` if the extraction was successful.
/// * `Err` if there was an error during extraction.
///
pub fn extract_to_dir_depending_on_content_type(
    content_type: &String,
    archive_path: &PathBuf,
    extract_to: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check the content type and extract accordingly
    match content_type.as_str() {
        "application/zip" => {
            println!("Extracting zip archive: {}", archive_path.display());
            let zip_file = File::open(archive_path)?;
            let mut archive = ZipArchive::new(zip_file)?;
            archive.extract(extract_to)?;
            println!(
                "Successfully extracted zip archive to {}",
                extract_to.to_string_lossy()
            );
        }
        "application/gzip" | "application/x-gtar" => {
            // Assuming this is tar.gz
            println!("Extracting tar.gz archive: {}", archive_path.display());
            let tar_gz_file = File::open(archive_path)?;
            let tar = GzDecoder::new(tar_gz_file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_to)?;
            println!(
                "Successfully extracted tar.gz archive to {}",
                extract_to.to_string_lossy()
            );
        }
        "application/x-xz" => {
            // Assuming this is tar.xz
            println!("Extracting tar.xz archive: {}", archive_path.display());
            let tar_xz_file = File::open(archive_path)?;
            let tar = XzDecoder::new(tar_xz_file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_to)?;
            println!(
                "Successfully extracted tar.xz archive to {}",
                extract_to.to_string_lossy()
            );
        }
        "application/x-bzip2" => {
            // Assuming this is tar.bz2
            println!("Extracting tar.bz2 archive: {}", archive_path.display());
            let tar_bz2_file = File::open(archive_path)?;
            let tar = BzDecoder::new(tar_bz2_file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_to)?;
            println!(
                "Successfully extracted tar.bz2 archive to {}",
                extract_to.to_string_lossy()
            );
        }
        "application/x-tar" => {
            // Assuming this is tar
            println!("Extracting tar archive: {}", archive_path.display());
            let tar_file = File::open(archive_path)?;
            let mut archive = Archive::new(tar_file);
            archive.unpack(extract_to)?;
            println!(
                "Successfully extracted tar archive to {}",
                extract_to.to_string_lossy()
            );
        }
        // TODO: 7z-support is experimental because untested
        "application/x-7z-compressed" => {
            // Assuming this is 7z
            println!("Extracting 7z archive: {}", archive_path.display());
            sevenz_rust2::decompress_file(archive_path, extract_to).expect("complete");
            println!(
                "Successfully extracted 7z archive to {}",
                extract_to.to_string_lossy()
            );
        }
        _ => {
            // Consider returning an error instead of just printing
            // return Err(format!("Unsupported content type for extraction: {}", content_type).into());
            println!("Unsupported content type for extraction: {}", content_type);
            std::process::exit(109);
        }
    }
    Ok(())
}
