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

// Magic byte signatures for archive format validation
//
// note: no need to support other zip magic bytes as this is more than enough
// for zip files used in software distribution.
const ZIP_MAGIC: &[u8] = &[0x50, 0x4B, 0x03, 0x04]; // "PK\x03\x04"
const GZIP_MAGIC: &[u8] = &[0x1F, 0x8B]; // gzip
const XZ_MAGIC: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]; // "\xfd7zXZ\x00"
const BZIP2_MAGIC: &[u8] = &[0x42, 0x5A, 0x68]; // "BZh"
const TAR_MAGIC_OFFSET: usize = 257;
const TAR_MAGIC: &[u8] = b"ustar";
const SEVENZ_MAGIC: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]; // 7z signature

// Fallback directory name for extracted files
const OUTPUT_DIR: &str = "output";

#[derive(Debug, PartialEq)]
enum ArchiveFormat {
    Zip,
    TarGz,
    TarXz,
    TarBz2,
    Tar,
    Gz,
    Xz,
    Bz2,
    SevenZ,
    Unknown,
}

/// Validates archive format by checking magic bytes against expected format
fn validate_magic_bytes(archive_path: &Path, expected_format: &ArchiveFormat) -> bool {
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
        ArchiveFormat::Zip => buffer.starts_with(ZIP_MAGIC),
        ArchiveFormat::TarGz | ArchiveFormat::Gz => buffer.starts_with(GZIP_MAGIC),
        ArchiveFormat::TarXz | ArchiveFormat::Xz => buffer.starts_with(XZ_MAGIC),
        ArchiveFormat::TarBz2 | ArchiveFormat::Bz2 => buffer.starts_with(BZIP2_MAGIC),
        ArchiveFormat::Tar => {
            // Check for tar magic at offset 257
            bytes_read > TAR_MAGIC_OFFSET + TAR_MAGIC.len()
                && &buffer[TAR_MAGIC_OFFSET..TAR_MAGIC_OFFSET + TAR_MAGIC.len()] == TAR_MAGIC
        }
        ArchiveFormat::SevenZ => buffer.starts_with(SEVENZ_MAGIC),
        ArchiveFormat::Unknown => false,
    }
}

/// Determines archive format from file extension
fn get_archive_format_from_extension(archive_path: &Path) -> ArchiveFormat {
    let extension = utils::get_file_extension(archive_path);
    match extension {
        // Multi-part extensions first (tar.xxx)
        "tar.gz" | "tgz" => ArchiveFormat::TarGz,
        "tar.xz" | "txz" => ArchiveFormat::TarXz,
        "tar.bz2" | "tbz" | "tbz2" => ArchiveFormat::TarBz2,
        // Single extensions
        "zip" => ArchiveFormat::Zip,
        "gz" => ArchiveFormat::Gz,
        "xz" => ArchiveFormat::Xz,
        "bz2" => ArchiveFormat::Bz2,
        "tar" => ArchiveFormat::Tar,
        "7z" => ArchiveFormat::SevenZ,
        _ => ArchiveFormat::Unknown,
    }
}

/// Determines the validated archive format using extension + magic byte validation
fn determine_validated_archive_format(archive_path: &Path) -> ArchiveFormat {
    let format_from_extension = get_archive_format_from_extension(archive_path);

    // For unknown extensions, we can't validate
    if format_from_extension == ArchiveFormat::Unknown {
        error!("Unsupported file extension for: {}", archive_path.display());
        return ArchiveFormat::Unknown;
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
        ArchiveFormat::Unknown
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
    let archive_format: ArchiveFormat = determine_validated_archive_format(archive_path);

    match archive_format {
        ArchiveFormat::Zip => {
            debug!("Extracting zip archive: {}", archive_path.display());
            let zip_file = File::open(archive_path)?;
            let mut archive = ZipArchive::new(zip_file)?;
            archive.extract(extract_to)?;
            debug!(
                "Successfully extracted zip archive to {}",
                extract_to.display()
            );
        }
        ArchiveFormat::TarGz => {
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
        ArchiveFormat::TarXz => {
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
        ArchiveFormat::TarBz2 => {
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
        ArchiveFormat::Tar => {
            debug!("Extracting tar archive: {}", archive_path.display());
            let tar_file = File::open(archive_path)?;
            let mut archive = Archive::new(tar_file);
            archive.unpack(extract_to)?;
            debug!(
                "Successfully extracted tar archive to {}",
                extract_to.display()
            );
        }
        ArchiveFormat::Gz => {
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
        ArchiveFormat::Xz => {
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
        ArchiveFormat::Bz2 => {
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
        ArchiveFormat::SevenZ => {
            debug!("Extracting 7z archive: {}", archive_path.display());
            sevenz_rust2::decompress_file(archive_path, extract_to).expect("complete");
            debug!(
                "Successfully extracted 7z archive to {}",
                extract_to.display()
            );
        }
        ArchiveFormat::Unknown => {
            std::process::exit(109);
        }
    }
    Ok(())
}
