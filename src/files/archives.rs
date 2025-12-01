use crate::files::magic::*;
use crate::files::utils::get_file_extension;
use crate::models::binary_container::BinaryContainer;

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

/// Validate archive format by checking magic bytes against expected format
fn validate_format_against_magic_bytes(
    archive_path: &Path,
    expected_format: &BinaryContainer,
) -> bool {
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

/// Determine archive format from file extension
fn get_archive_format_from_extension(archive_path: &Path) -> BinaryContainer {
    let extension: String = get_file_extension(archive_path).to_lowercase();
    match extension.as_str() {
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

/// Validates and determines the archive format of a file using a two-step verification process.
///
/// This function provides a secure way to identify archive formats by performing both
/// extension-based detection and magic byte validation. This dual-validation approach
/// prevents format spoofing attacks where a file might have a misleading extension.
///
/// # Validation Process
///
/// 1. **Extension Detection**: First attempts to determine the archive format from the file
///    extension (e.g., `.zip`, `.tar.gz`, `.7z`). If the extension is not recognized as a
///    supported archive format, the function immediately returns `BinaryContainer::Unknown`.
///
/// 2. **Magic Byte Validation**: For recognized extensions, validates that the file's magic
///    bytes (file signature) match the expected format. This ensures the file is actually
///    of the claimed type and not corrupted or misnamed.
///
/// # Security Considerations
///
/// - **Format Spoofing Prevention**: By validating magic bytes against the extension, this
///   function prevents malicious files from being processed with incorrect handlers.
/// - **Corruption Detection**: Files that appear corrupted (mismatched magic bytes) are
///   rejected and logged as errors.
/// - **Conservative Approach**: When validation fails, the function returns `Unknown` rather
///   than attempting to guess the format, prioritizing security over convenience.
///
/// # Supported Archive Formats
///
/// The function recognizes and validates the following archive formats:
///
/// - **ZIP** (`.zip`): ZIP archive format (PK magic bytes)
/// - **TAR** (`.tar`): Uncompressed TAR archive (POSIX ustar format)
/// - **TAR.GZ/TGZ** (`.tar.gz`, `.tgz`): GZip-compressed TAR archive
/// - **TAR.XZ/TXZ** (`.tar.xz`, `.txz`): XZ-compressed TAR archive
/// - **TAR.BZ2/TBZ/TBZ2** (`.tar.bz2`, `.tbz`, `.tbz2`): BZip2-compressed TAR archive
/// - **GZ** (`.gz`): Standalone GZip-compressed file (not commonly used for distribution)
/// - **XZ** (`.xz`): Standalone XZ-compressed file (not commonly used for distribution)
/// - **BZ2** (`.bz2`): Standalone BZip2-compressed file (not commonly used for distribution)
/// - **7Z** (`.7z`): 7-Zip archive format
///
/// # Arguments
///
/// * `archive_path` - A path reference to the archive file to be validated. The path must
///   point to an accessible file on the filesystem, as the function needs to read the file's
///   magic bytes for validation.
///
/// # Returns
///
/// Returns a `Result<BinaryContainer, Box<dyn std::error::Error>>` indicating the validated archive format:
///
/// - **Specific Format** (e.g., `BinaryContainer::Zip`, `BinaryContainer::TarGz`): Returned
///   when both the extension is recognized and the magic bytes validation passes.
/// - **`BinaryContainer::Unknown`**: Returned in the following cases:
///   - The file extension is not recognized as a supported archive format
///   - The file cannot be opened or read
///   - The magic bytes don't match the expected format (possible corruption or format spoofing)
///   - The file is too small to contain valid magic bytes
///
/// # Error Handling and Logging
///
/// * `Err(Box<dyn std::error::Error>)` if the archive format is unknown.
/// * `Err(std::process::exit(109))` if the archive format is unknown.
///
/// # Performance Considerations
///
/// - **I/O Operations**: This function performs file I/O to read magic bytes (up to 512 bytes),
///   which may add latency for network-mounted filesystems.
/// - **Buffered Reading**: Only reads the minimum bytes necessary for validation (512 bytes
///   maximum) to minimize overhead.
/// - **Early Exit**: Returns immediately for unrecognized extensions without file I/O.
///
/// # Notes
///
/// - Multi-part extensions (e.g., `.tar.gz`) are recognized and handled correctly
/// - Case-insensitive extension matching is performed
/// - The function is intentionally conservative: ambiguous cases result in `Unknown`
/// - TAR magic bytes are located at offset 257 in the file (POSIX ustar format)
///
pub fn get_validated_archive_format(
    archive_path: &Path,
) -> Result<BinaryContainer, Box<dyn std::error::Error>> {
    let format_from_extension = get_archive_format_from_extension(archive_path);

    if format_from_extension == BinaryContainer::Unknown
        || !validate_format_against_magic_bytes(archive_path, &format_from_extension)
    {
        let msg: &str = "Unsupported file extension or file is corrupted";
        error!("{}", msg);
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            msg,
        )))
    } else {
        debug!(
            "Archive format {:?} is valid for file {}",
            format_from_extension,
            archive_path.display()
        );
        Ok(format_from_extension)
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
/// * `Err(Box<dyn std::error::Error>)` if there was an error during extraction.
/// * `Err(std::process::exit(109))` if the archive format is unknown.
///
pub fn extract_to_dir(
    archive_path: &PathBuf,
    extract_to: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let archive_format: BinaryContainer = get_validated_archive_format(archive_path)
        .unwrap_or_else(|e| {
            error!(
                "Error while validating archive format of {}: {}",
                archive_path.display(),
                e
            );
            std::process::exit(109);
        });

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

#[cfg(test)]
mod tests;
