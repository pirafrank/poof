use crate::files::magic::*;
use crate::files::utils::get_file_extension;
use crate::models::binary_container::BinaryContainer;

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use log::debug;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;
use xz2::read::XzDecoder;
use zip::read::ZipArchive;

use anyhow::{bail, Context, Result};

// Fallback directory name for extracted files
const OUTPUT_DIR: &str = "output";

/// Validates an archive file's magic bytes against its expected format.
///
/// This function reads the first 512 bytes of a file and checks whether the magic bytes
/// (file signature) match the expected archive format. This validation is crucial for
/// detecting file corruption, format spoofing, or misnamed files.
///
/// # Arguments
///
/// * `archive_path` - Path to the archive file to validate
/// * `expected_format` - The expected archive format to validate against
///
/// # Returns
///
/// * `true` if the magic bytes match the expected format
/// * `false` if:
///   - The file cannot be opened or read
///   - The file is too small (less than 4 bytes)
///   - The magic bytes don't match the expected format
///   - The expected format is `BinaryContainer::Unknown`
///
/// # Magic Byte Validation
///
/// Different archive formats are validated as follows:
/// - **ZIP**: Checks for "PK" signature at the start
/// - **GZIP** (GZ, TAR.GZ): Checks for GZIP magic bytes at the start
/// - **XZ** (XZ, TAR.XZ): Checks for XZ magic bytes at the start
/// - **BZIP2** (BZ2, TAR.BZ2): Checks for BZIP2 magic bytes at the start
/// - **TAR**: Checks for "ustar" signature at offset 257 (POSIX tar format)
/// - **7Z**: Checks for 7-Zip signature at the start
///
/// # Notes
///
/// - Reads up to 512 bytes to accommodate TAR magic bytes at offset 257
/// - Does not perform complete file validation, only checks magic bytes
/// - This is an internal helper function used by `get_validated_archive_format`
///
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

/// Determines the archive format based on the file's extension.
///
/// This function performs case-insensitive matching of file extensions to identify
/// the archive format. It correctly handles both simple extensions (e.g., `.zip`) and
/// multi-part extensions (e.g., `.tar.gz`).
///
/// # Arguments
///
/// * `archive_path` - Path to the archive file whose format should be determined
///
/// # Returns
///
/// Returns a `BinaryContainer` enum variant representing the detected archive format:
/// - `BinaryContainer::Zip` for `.zip` files
/// - `BinaryContainer::TarGz` for `.tar.gz` or `.tgz` files
/// - `BinaryContainer::TarXz` for `.tar.xz` or `.txz` files
/// - `BinaryContainer::TarBz2` for `.tar.bz2`, `.tbz`, or `.tbz2` files
/// - `BinaryContainer::Gz` for standalone `.gz` files
/// - `BinaryContainer::Xz` for standalone `.xz` files
/// - `BinaryContainer::Bz2` for standalone `.bz2` files
/// - `BinaryContainer::Tar` for `.tar` files
/// - `BinaryContainer::SevenZ` for `.7z` files
/// - `BinaryContainer::Unknown` for unrecognized extensions
///
/// # Extension Handling
///
/// - **Case Insensitive**: Extensions are converted to lowercase before matching
/// - **Multi-part Extensions**: Recognized before single extensions (e.g., `.tar.gz` takes precedence)
/// - **Common Abbreviations**: Supports standard abbreviations (`.tgz`, `.txz`, `.tbz`, `.tbz2`)
///
/// # Notes
///
/// - This function only performs extension-based detection without validating the file content
/// - For production use, combine with `validate_format_against_magic_bytes` to prevent spoofing
/// - The function uses the `get_file_extension` utility for extraction
///
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
/// # Errors
///
/// Returns an error in the following cases:
/// - The file extension is not recognized as a supported archive format
/// - The file's magic bytes don't match the expected format (indicating corruption or spoofing)
/// - The file cannot be opened or read during validation
/// - The file is too small to contain valid magic bytes
///
/// # Performance Considerations
///
/// - **I/O Operations**: Performs file I/O to read magic bytes (up to 512 bytes),
///   which may add latency for network-mounted filesystems
/// - **Buffered Reading**: Only reads the minimum bytes necessary for validation (512 bytes
///   maximum) to minimize overhead
/// - **Early Exit**: Returns immediately for unrecognized extensions without file I/O
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use crate::files::archives::get_validated_archive_format;
/// use crate::models::binary_container::BinaryContainer;
///
/// let archive_path = Path::new("/path/to/archive.tar.gz");
/// match get_validated_archive_format(archive_path) {
///     Ok(BinaryContainer::TarGz) => println!("Valid tar.gz archive"),
///     Ok(format) => println!("Archive format: {:?}", format),
///     Err(e) => eprintln!("Validation failed: {}", e),
/// }
/// ```
///
/// # Notes
///
/// - Multi-part extensions (e.g., `.tar.gz`) are recognized and handled correctly
/// - Case-insensitive extension matching is performed
/// - The function is intentionally conservative: ambiguous cases result in errors
/// - TAR magic bytes are located at offset 257 in the file (POSIX ustar format)
///
pub fn get_validated_archive_format(archive_path: &Path) -> Result<BinaryContainer> {
    let format_from_extension = get_archive_format_from_extension(archive_path);

    if format_from_extension == BinaryContainer::Unknown
        || !validate_format_against_magic_bytes(archive_path, &format_from_extension)
    {
        bail!("Unsupported file extension or file is corrupted");
    } else {
        debug!(
            "Archive format {:?} is valid for file {}",
            format_from_extension,
            archive_path.display()
        );
        Ok(format_from_extension)
    }
}

/// Extracts an archive file to a specified directory with format validation.
///
/// This function provides a unified interface for extracting various archive formats.
/// It automatically detects the archive format based on file extension and validates
/// it against magic bytes before extraction, ensuring security and preventing corruption.
///
/// # Supported Archive Formats
///
/// The function supports the following archive formats with their respective handlers:
///
/// - **ZIP** (`.zip`): Uses the `zip` crate for extraction
/// - **TAR** (`.tar`): Uncompressed TAR archives using the `tar` crate
/// - **TAR.GZ/TGZ** (`.tar.gz`, `.tgz`): GZip-compressed TAR archives
/// - **TAR.XZ/TXZ** (`.tar.xz`, `.txz`): XZ-compressed TAR archives
/// - **TAR.BZ2/TBZ/TBZ2** (`.tar.bz2`, `.tbz`, `.tbz2`): BZip2-compressed TAR archives
/// - **GZ** (`.gz`): Standalone GZip-compressed files (uncommon for distribution)
/// - **XZ** (`.xz`): Standalone XZ-compressed files (uncommon for distribution)
/// - **BZ2** (`.bz2`): Standalone BZip2-compressed files (uncommon for distribution)
/// - **7Z** (`.7z`): 7-Zip archives using the `sevenz-rust2` crate
///
/// # Arguments
///
/// * `archive_path` - Path to the archive file to extract. The file must exist and be readable.
/// * `extract_to` - Target directory where the archive contents will be extracted. The directory
///   will be created if it doesn't exist (for standalone compressed files).
///
/// # Returns
///
/// * `Ok(())` if the extraction completed successfully
/// * `Err(anyhow::Error)` if an error occurred during:
///   - Archive format validation
///   - File opening or reading
///   - Archive extraction
///   - Directory creation
///
/// # Errors
///
/// This function can return errors in the following cases:
/// - **Unsupported Format**: The file extension is not recognized or magic bytes don't match
/// - **File Not Found**: The archive file doesn't exist or isn't readable
/// - **Extraction Failure**: The archive is corrupted or incomplete
/// - **Permission Denied**: Insufficient permissions to read the archive or write to the target
/// - **I/O Errors**: Disk full, filesystem errors, etc.
///
/// # Behavior Details
///
/// ## Directory Creation
/// - For TAR and ZIP archives, the target directory should exist or be creatable
/// - For standalone compressed files (GZ, XZ, BZ2), the function creates the target directory
///
/// ## Output File Naming
/// - **TAR/ZIP archives**: Preserves the internal directory structure
/// - **Standalone compressed files**: Creates a file in the target directory using the archive's
///   stem name (e.g., `file.txt.gz` → `file.txt` in the target directory)
/// - **Fallback**: If the stem name cannot be determined, uses `OUTPUT_DIR` constant
///
/// ## Logging
/// - Debug logs are emitted before and after extraction for each format
/// - Logs include the archive path and extraction target for traceability
///
/// # Security Considerations
///
/// - **Format Validation**: All archives are validated via `get_validated_archive_format`
///   before extraction to prevent format spoofing attacks
/// - **Magic Byte Verification**: Ensures the file content matches its claimed format
/// - **Path Traversal**: Archive extraction libraries handle path traversal protection
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use crate::files::archives::extract_to_dir;
///
/// let archive = PathBuf::from("/downloads/release.tar.gz");
/// let target = PathBuf::from("/tmp/extracted");
///
/// match extract_to_dir(&archive, &target) {
///     Ok(()) => println!("Extraction successful"),
///     Err(e) => eprintln!("Extraction failed: {}", e),
/// }
/// ```
///
/// # Notes
///
/// - The 7Z extraction uses `expect("complete")` which will panic on failure
/// - Standalone compressed files (GZ, XZ, BZ2) are rarely used for software distribution
/// - Multi-part extensions (e.g., `.tar.gz`) are correctly identified before single extensions
///
pub fn extract_to_dir(archive_path: &PathBuf, extract_to: &PathBuf) -> Result<()> {
    let archive_format: BinaryContainer =
        get_validated_archive_format(archive_path).with_context(|| {
            format!(
                "Error while validating archive format of {}",
                archive_path.display()
            )
        })?;

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
        _ => bail!("Unsupported archive format: {:?}", archive_format),
    }
    Ok(())
}

#[cfg(test)]
mod tests;
