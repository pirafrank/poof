use crate::files::magic::{
    BZIP2_MAGIC, GZIP_MAGIC, SEVENZ_MAGIC, TAR_MAGIC, TAR_MAGIC_OFFSET, XZ_MAGIC, ZIP_MAGIC,
};
use crate::models::binary_container::BinaryContainer;
use crate::utils;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use log::{debug, error};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;
use xz2::read::XzDecoder;
use zip::ZipArchive;

// Fallback directory name for extracted files
const OUTPUT_DIR: &str = "output";

/// Validates archive format by checking magic bytes against expected format
fn validate_magic_bytes(archive_path: &Path, expected_format: &BinaryContainer) -> bool {
    let mut file = match File::open(archive_path) {
        Ok(f) => f,
        Err(_) => return false,
    };

    let mut buffer = [0u8; 512]; // Read enough for tar magic at offset 257
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };

    // makes no sense to have files smaller than magic bytes size
    if bytes_read < 4 {
        return false;
    }

    match expected_format {
        BinaryContainer::Zip => buffer.starts_with(ZIP_MAGIC),
        BinaryContainer::TarGz | BinaryContainer::Gz => buffer.starts_with(GZIP_MAGIC),
        BinaryContainer::TarXz | BinaryContainer::Xz => buffer.starts_with(XZ_MAGIC),
        BinaryContainer::TarBz2 | BinaryContainer::Bz2 => buffer.starts_with(BZIP2_MAGIC),
        BinaryContainer::Tar => {
            // Check for tar magic at offset 257
            bytes_read > TAR_MAGIC_OFFSET + TAR_MAGIC.len()
                && &buffer[TAR_MAGIC_OFFSET..TAR_MAGIC_OFFSET + TAR_MAGIC.len()] == TAR_MAGIC
        }
        BinaryContainer::SevenZ => buffer.starts_with(SEVENZ_MAGIC),
        BinaryContainer::Unknown => false,
    }
}

/// Determines archive format from file extension
fn get_archive_format_from_extension(archive_path: &Path) -> BinaryContainer {
    let extension = utils::get_file_extension(archive_path);
    match extension {
        // Multi-part extensions first (tar.xxx)
        "tar.gz" | "tgz" => BinaryContainer::TarGz,
        "tar.xz" | "txz" => BinaryContainer::TarXz,
        "tar.bz2" | "tbz" | "tbz2" => BinaryContainer::TarBz2,
        // Single extensions
        "zip" => BinaryContainer::Zip,
        "gz" => BinaryContainer::Gz,
        "xz" => BinaryContainer::Xz,
        "bz2" => BinaryContainer::Bz2,
        "tar" => BinaryContainer::Tar,
        "7z" => BinaryContainer::SevenZ,
        _ => BinaryContainer::Unknown,
    }
}

/// Determines the validated archive format using extension + magic byte validation
fn determine_validated_archive_format(archive_path: &Path) -> BinaryContainer {
    let format_from_extension = get_archive_format_from_extension(archive_path);

    // For unknown extensions, we can't validate
    if format_from_extension == BinaryContainer::Unknown {
        error!("Unsupported file extension for: {}", archive_path.display());
        return BinaryContainer::Unknown;
    }

    // Validate the extension against magic bytes
    if validate_magic_bytes(archive_path, &format_from_extension) {
        debug!(
            "Archive format {:?} is valid for file {}",
            format_from_extension,
            archive_path.display()
        );
        format_from_extension
    } else {
        error!(
            "Cannot validate archive format {:?} for file: {}. Is it a corrupted file?",
            format_from_extension,
            archive_path.display()
        );
        // Could try to detect actual format here, but for security we'll mark as unknown
        BinaryContainer::Unknown
    }
}

/// Extracts an archive to a specified directory based on validated file extension.
/// Supports zip, tar.gz, tar.xz, tar.bz2, and other common archive formats.
/// The format is determined by file extension and validated against magic bytes.
///
/// # Arguments
/// * `archive_path` - The path to the archive file.
/// * `extract_to` - The directory where the archive will be extracted.
///
/// # Returns
/// * `Ok(())` if the extraction was successful.
/// * `Err` if there was an error during extraction.
///
pub fn extract_to_dir(
    archive_path: &PathBuf,
    extract_to: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let archive_format: BinaryContainer = determine_validated_archive_format(archive_path);

    match archive_format {
        BinaryContainer::Zip => {
            debug!("Extracting zip archive: {}", archive_path.display());
            let zip_file = File::open(archive_path)?;
            let mut archive = ZipArchive::new(zip_file)?;
            archive.extract(extract_to)?;
            debug!(
                "Successfully extracted zip archive to {}",
                extract_to.display()
            );
        }
        BinaryContainer::TarGz => {
            debug!("Extracting tar.gz archive: {}", archive_path.display());
            let tar_gz_file = File::open(archive_path)?;
            let tar = GzDecoder::new(tar_gz_file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_to)?;
            debug!(
                "Successfully extracted tar.gz archive to {}",
                extract_to.display()
            );
        }
        BinaryContainer::TarXz => {
            debug!("Extracting tar.xz archive: {}", archive_path.display());
            let tar_xz_file = File::open(archive_path)?;
            let tar = XzDecoder::new(tar_xz_file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_to)?;
            debug!(
                "Successfully extracted tar.xz archive to {}",
                extract_to.display()
            );
        }
        BinaryContainer::TarBz2 => {
            debug!("Extracting tar.bz2 archive: {}", archive_path.display());
            let tar_bz2_file = File::open(archive_path)?;
            let tar = BzDecoder::new(tar_bz2_file);
            let mut archive = Archive::new(tar);
            archive.unpack(extract_to)?;
            debug!(
                "Successfully extracted tar.bz2 archive to {}",
                extract_to.display()
            );
        }
        BinaryContainer::Tar => {
            debug!("Extracting tar archive: {}", archive_path.display());
            let tar_file = File::open(archive_path)?;
            let mut archive = Archive::new(tar_file);
            archive.unpack(extract_to)?;
            debug!(
                "Successfully extracted tar archive to {}",
                extract_to.display()
            );
        }
        BinaryContainer::Gz => {
            // Plain gzip file (not tar.gz) - not really used for software distribution
            debug!("Extracting gz archive: {}", archive_path.display());
            let gz_file = File::open(archive_path)?;
            let mut decoder = GzDecoder::new(gz_file);
            let output_path = extract_to.join(
                archive_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(OUTPUT_DIR),
            );
            std::fs::create_dir_all(extract_to)?;
            let mut output_file = File::create(&output_path)?;
            std::io::copy(&mut decoder, &mut output_file)?;
            debug!(
                "Successfully extracted gz archive to {}",
                output_path.display()
            );
        }
        BinaryContainer::Xz => {
            // Plain xz file (not tar.xz) - not really used for software distribution
            debug!("Extracting xz archive: {}", archive_path.display());
            let xz_file = File::open(archive_path)?;
            let mut decoder = XzDecoder::new(xz_file);
            let output_path = extract_to.join(
                archive_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(OUTPUT_DIR),
            );
            std::fs::create_dir_all(extract_to)?;
            let mut output_file = File::create(&output_path)?;
            std::io::copy(&mut decoder, &mut output_file)?;
            debug!(
                "Successfully extracted xz archive to {}",
                output_path.display()
            );
        }
        BinaryContainer::Bz2 => {
            // Plain bzip2 file (not tar.bz2) - not really used for software distribution
            debug!("Extracting bz2 archive: {}", archive_path.display());
            let bz2_file = File::open(archive_path)?;
            let mut decoder = BzDecoder::new(bz2_file);
            let output_path = extract_to.join(
                archive_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(OUTPUT_DIR),
            );
            std::fs::create_dir_all(extract_to)?;
            let mut output_file = File::create(&output_path)?;
            std::io::copy(&mut decoder, &mut output_file)?;
            debug!(
                "Successfully extracted bz2 archive to {}",
                output_path.display()
            );
        }
        BinaryContainer::SevenZ => {
            debug!("Extracting 7z archive: {}", archive_path.display());
            sevenz_rust2::decompress_file(archive_path, extract_to).expect("complete");
            debug!(
                "Successfully extracted 7z archive to {}",
                extract_to.display()
            );
        }
        BinaryContainer::Unknown => {
            std::process::exit(109);
        }
    }
    Ok(())
}
