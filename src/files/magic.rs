use std::{fs::File, io::Read, path::Path};

// Magic number constants for file format detection

#[cfg(target_os = "macos")]
pub const MACHO_MAGIC_NUMBERS: &[[u8; 4]] = &[
    [0xFE, 0xED, 0xFA, 0xCE], // Mach-O 32-bit (little-endian)
    [0xFE, 0xED, 0xFA, 0xCF], // Mach-O 64-bit (little-endian)
    [0xCE, 0xFA, 0xED, 0xFE], // Mach-O 32-bit (big-endian)
    [0xCF, 0xFA, 0xED, 0xFE], // Mach-O 64-bit (big-endian)
    [0xCA, 0xFE, 0xBA, 0xBE], // Mach-O universal ('fat') binary (little-endian)
    [0xBE, 0xBA, 0xFE, 0xCA], // Mach-O universal ('fat') binary (big-endian)
];

#[cfg(target_os = "linux")]
// ELF magic number for Linux executables
pub const ELF_MAGIC: [u8; 4] = [0x7F, 0x45, 0x4C, 0x46]; // ELF

#[cfg(target_os = "windows")]
// PE magic number for Windows executables (MZ header)
pub const PE_MAGIC: [u8; 2] = [0x4D, 0x5A]; // MZ

// Archive format magic numbers
pub const ZIP_MAGIC: &[u8] = &[0x50, 0x4B, 0x03, 0x04]; // "PK\x03\x04"
pub const GZIP_MAGIC: &[u8] = &[0x1F, 0x8B]; // gzip
pub const XZ_MAGIC: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00]; // "\xfd7zXZ\x00"
pub const BZIP2_MAGIC: &[u8] = &[0x42, 0x5A, 0x68]; // "BZh"
pub const TAR_MAGIC_OFFSET: usize = 257;
pub const TAR_MAGIC: &[u8] = b"ustar";
pub const SEVENZ_MAGIC: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C]; // 7z signature

#[cfg(target_os = "linux")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    // Linux expects ELF binaries
    buffer == &ELF_MAGIC // ELF
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

#[cfg(not(target_os = "windows"))]
pub fn is_exec_by_magic_number(path: &Path) -> bool {
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0u8; 4];
        if file.read_exact(&mut buffer).is_ok() {
            return is_exec_magic(&buffer);
        }
    }
    false
}

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
