//! Magic number constants and file-format detection utilities.
//!
//! These byte sequences are read from the start (or a fixed offset) of a file
//! to identify its format without relying on file extensions.

use anyhow::Result;
use std::env;
use std::io::{Read, Seek, SeekFrom};
use std::{fs::File, path::Path};

/// Unix shebang prefix (`#!`) used by interpreted scripts.
pub const SHEBANG_MAGIC: &[u8] = &[0x23, 0x21]; // "#!"

/// Mach-O magic numbers for 32-bit, 64-bit, and universal ('fat') binaries (macOS only).
#[cfg(target_os = "macos")]
pub const MACHO_MAGIC_NUMBERS: &[[u8; 4]] = &[
    [0xFE, 0xED, 0xFA, 0xCE], // Mach-O 32-bit (big-endian)
    [0xFE, 0xED, 0xFA, 0xCF], // Mach-O 64-bit (big-endian)
    [0xCE, 0xFA, 0xED, 0xFE], // Mach-O 32-bit (little-endian)
    [0xCF, 0xFA, 0xED, 0xFE], // Mach-O 64-bit (little-endian)
    [0xCA, 0xFE, 0xBA, 0xBE], // Mach-O universal ('fat') binary (big-endian)
    [0xBE, 0xBA, 0xFE, 0xCA], // Mach-O universal ('fat') binary (little-endian)
];

/// ELF magic number identifying Linux (and most Unix) executables (Linux only).
#[cfg(target_os = "linux")]
pub const ELF_MAGIC: [u8; 4] = [0x7F, 0x45, 0x4C, 0x46]; // ELF

/// PE/MZ magic number identifying Windows executables (Windows only).
#[cfg(target_os = "windows")]
pub const PE_MAGIC: [u8; 2] = [0x4D, 0x5A]; // MZ

/// ZIP archive magic number (`PK\x03\x04`).
pub const ZIP_MAGIC: &[u8] = &[0x50, 0x4B, 0x03, 0x04]; // "PK\x03\x04"
/// Gzip stream magic number.
pub const GZIP_MAGIC: &[u8] = &[0x1F, 0x8B]; // gzip
/// Zstandard frame magic number.
pub const ZSTD_MAGIC: &[u8] = &[0x28, 0xB5, 0x2F, 0xFD]; // zstd
/// XZ stream magic number.
pub const XZ_MAGIC: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]; // "\xfd7zXZ\x00"
/// Bzip2 stream magic number (`BZh`).
pub const BZIP2_MAGIC: &[u8] = &[0x42, 0x5A, 0x68]; // "BZh"
/// Byte offset within a tar archive where the `ustar` magic string is located.
pub const TAR_MAGIC_OFFSET: usize = 257;
/// Tar POSIX magic string (`ustar`) found at [`TAR_MAGIC_OFFSET`].
pub const TAR_MAGIC: &[u8] = b"ustar";
/// 7-Zip archive signature bytes.
pub const SEVENZ_MAGIC: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]; // 7z signature

/// Returns `true` if the first four bytes of `buffer` match the ELF magic number.
#[cfg(target_os = "linux")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    buffer == &ELF_MAGIC
}

#[cfg(target_os = "windows")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    // Windows expects PE binaries (MZ header).
    // Checking only the first two bytes because the other two may change,
    // as they depend on the DOS stub.
    buffer[..2] == core::magic::PE_MAGIC
}

#[cfg(target_os = "macos")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    // macOS expects Mach-O formats
    MACHO_MAGIC_NUMBERS.contains(buffer)
}

/// Return `true` when the file at `path` appears to be an executable binary.
///
/// Detection is based on magic bytes at the start of the file rather than
/// file-name extensions. On non-Windows platforms both shebang scripts (`#!`)
/// and native binary formats (ELF on Linux, Mach-O on macOS) are recognised.
/// On Windows only `.exe` files with a valid PE/MZ header are accepted.
#[cfg(not(target_os = "windows"))]
pub fn is_exec_by_magic_number(path: &Path) -> bool {
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0u8; 4];
        if file.read_exact(&mut buffer).is_ok() {
            if buffer.starts_with(SHEBANG_MAGIC) {
                return true;
            }
            return is_exec_magic(&buffer);
        }
    }
    false
}

/// Return `true` when the file at `path` appears to be an executable binary (Windows variant).
///
/// Only files with an `.exe` extension whose first two bytes match the PE/MZ
/// magic number are considered executables.
#[cfg(target_os = "windows")]
pub fn is_exec_by_magic_number(path: &Path) -> bool {
    // We need to first check the file extension for Windows binaries,
    // as it uses the PE format (MZ header) for file types other than
    // .exe (e.g. .dll, .sys, etc.).
    // Then we check the first two bytes of the .exe file because the
    // other two may change (they depend on the DOS stub).
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    if extension != "exe" {
        return false;
    }
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0u8; 4];
        if file.read_exact(&mut buffer).is_ok() {
            return is_exec_magic(&buffer);
        }
    }
    false
}

/// Return `true` when the file at `file_path` appears to be a binary for the current architecture.
///
/// The function checks the machine type of the binary to determine if it is for the current architecture.
/// The function returns `true` if the binary is for the current architecture, `false` otherwise.
/// The function returns an error if the file cannot be opened or read.
///
/// # Arguments
///
/// * `file_path` - The path to the file to check.
///
/// # Returns
///
/// * `true` if the binary is for the current architecture, `false` otherwise.
pub fn is_exec_for_current_arch(file_path: &Path) -> Result<bool> {
    #[cfg(target_os = "linux")]
    {
        // check if the file is an ELF file
        if !is_exec_by_magic_number(file_path) {
            return Ok(false);
        }

        let mut file = File::open(file_path)?;

        // Seek to the e_machine field at offset 0x12
        file.seek(SeekFrom::Start(0x12))?;
        let mut e_machine = [0u8; 2];
        file.read_exact(&mut e_machine)?;

        // Read as little-endian (standard for both AMD64 and standard ARM64 Linux)
        let machine_type = u16::from_le_bytes(e_machine);

        let is_match = matches!(
            (env::consts::ARCH, machine_type),
            ("x86_64", 0x3E)
                | ("aarch64", 0xB7)
                | ("i686", 0x3E)
                | ("armv7", 0x3E)
                | ("riscv64", 0xF3)
                | ("powerpc64le", 0xB7)
                | ("s390x", 0x15)
                | ("loongarch64", 0xB7)
        );

        Ok(is_match)
    }

    #[cfg(target_os = "macos")]
    {
        return Ok(is_exec_by_magic_number(file_path));
    }

    #[cfg(target_os = "windows")]
    {
        return Ok(is_exec_by_magic_number(file_path));
    }
}
