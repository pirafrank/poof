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
    // Both aarch64 and x86_64 are little-endian
    [0xCF, 0xFA, 0xED, 0xFE], // Mach-O 64-bit (little-endian)
    // Mach-O universal ('fat') binary is always big-endian on disk.
    [0xCA, 0xFE, 0xBA, 0xBE], // Mach-O universal ('fat') binary (big-endian)
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
    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; 4];
    if file.read_exact(&mut buffer).is_err() {
        return Ok(false);
    }

    // Check if the file is a shebang script,
    // we won't check the rest of the file in such case.
    // It's a valid use case for a shebang script to be an executable.
    if buffer.starts_with(SHEBANG_MAGIC) {
        return Ok(true);
    }

    // Note: it's likely ok to make multiple seeks for the sake of readability.
    //       Performance-wise it's not a big deal since we're reading small chunks of data.
    //       Data is likely to be already in memory as OS puts everything in memory
    //       on first read since its page cache is 4kb. It's zero I/O cost.

    #[cfg(target_os = "linux")]
    {
        // Check if the file is an ELF file
        if !is_exec_magic(&buffer) {
            return Ok(false);
        }

        // No need to check EI_OSABI at offset 0x07.
        // Most Linux binaries are tagged 0x00, very few use 0x03 (Linux-specific).
        // Also, we are going to support *BSD which may often use 0x00 instead of
        // more specific variants (OpenBSD, FreeBSD, NetBSD) for compatibility reasons.
        // If we got here it's likely we downloaded the correct file thanks to previous checks.
        // Docs: https://refspecs.linuxfoundation.org/elf/gabi4+/ch4.eheader.html

        // Read EI_DATA first (endianness discriminator at offset 0x05)
        file.seek(SeekFrom::Start(0x05))?;
        let mut ei_data = [0u8; 1];
        file.read_exact(&mut ei_data)?;

        // Check e_machine at offset 0x12 to confirm architecture compatibility
        file.seek(SeekFrom::Start(0x12))?;
        let mut e_machine = [0u8; 2];
        file.read_exact(&mut e_machine)?;

        // Read e_machine based on EI_DATA (e.g. s390x is big-endian)
        let machine_type = match ei_data[0] {
            1 => u16::from_le_bytes(e_machine), // ELFDATA2LSB
            2 => u16::from_be_bytes(e_machine), // ELFDATA2MSB
            _ => return Ok(false),
        };

        // Check if the machine type matches the current architecture.
        // Docs:
        // https://cr0mll.github.io/cyberclopaedia/Reverse%20Engineering/Binary%20Formats/ELF/The%20ELF%20Header.html
        // https://gist.github.com/x0nu11byt3/bcb35c3de461e5fb66173071a2379779
        // https://loongson.github.io/LoongArch-Documentation/LoongArch-ELF-ABI-EN.html
        //
        // Note: EM_386 (0x03) is correct for all of i386, i486, i586, and i686.
        //       poof targets i686 among these.
        let is_match = matches!(
            (env::consts::ARCH, machine_type),
            ("x86_64", 0x3E)             // EM_X86_64    =  62
                | ("aarch64", 0xB7)      // EM_AARCH64   = 183
                | ("x86", 0x03)          // EM_386       =   3
                | ("arm", 0x28)          // EM_ARM       =  40
                | ("riscv64", 0xF3)      // EM_RISCV     = 243
                | ("powerpc64", 0x15)    // EM_PPC64     =  21
                | ("s390x", 0x16)        // EM_S390      =  22
                | ("loongarch64", 0x102) // EM_LOONGARCH = 258
        );

        // Note: to save reading bytes, we do not check for EI_CLASS.
        //       It is needed only for riscv32/64 and loongarch32/64
        //       where 32 and 64 bits share the same ELF header.
        //       Yet it is not needed since there's no real software
        //       out there that runs on 32-bit riscv or loongarch,
        //       and even if there was, it would be stopped by other
        //       checks before we even get to this point.
        Ok(is_match)
    }

    #[cfg(target_os = "macos")]
    {
        // Check if the file is a Mach-O binary
        if !is_exec_magic(&buffer) {
            return Ok(false);
        }

        // Check if the cputype matches the current architecture.
        // On Mac we have two possible formats: fat binary and 'thin' (single-arch) binary.
        // Docs:
        // https://github.com/apple-oss-distributions/xnu/blob/main/EXTERNAL_HEADERS/mach-o/fat.h
        // https://github.com/apple-oss-distributions/xnu/blob/main/osfmk/mach/machine.h
        match buffer {
            // Fat binary (32 bit) - header is always big-endian on disk regardless of host CPU.
            // Iterate the fat_arch table as many times as the nfat_arch field indicates.
            // each entry is 20 bytes, cputype is the first 4 bytes (BE) of the entry.
            // Note: we don't care about fat64 binaries, no one would really release a pre-built
            //       binary so big to require fat64 (> 4GB).
            [0xCA, 0xFE, 0xBA, 0xBE] => {
                // 4 bytes after the magic number to know how many architectures are in the file
                file.seek(SeekFrom::Start(4))?;
                let mut n = [0u8; 4];
                file.read_exact(&mut n)?;
                let nfat_arch = u32::from_be_bytes(n);

                // Check that the file is long enough to contain:
                // - the magic number (4 bytes)
                // - the nfat_arch field (4 bytes)
                // - the fat_arch table (20 bytes per entry * nfat_arch)
                let file_len = file.metadata()?.len();
                // Total table size: the table is 20 bytes per entry,
                // so we multiply the number of entries by 20.
                let table_bytes: u64 = nfat_arch as u64 * 20;
                let minimum_required_file_len = 8 + table_bytes;
                if file_len < minimum_required_file_len {
                    return Ok(false);
                }

                // Iterate the fat_arch table to find a matching cputype.
                // This because fat binaries do not enforce any specif order of the architectures.
                // This is different from ELF, where the order is enforced.
                // If we find a match, return true.
                for _ in 0..nfat_arch {
                    // Read entries in the fat_arch table, one at time.
                    // Each entry is 20 bytes wide, cputype is the first 4 (BE).
                    // Check fat_arch struct definition, which holds 5 fields of 4 bytes each.
                    let mut entry = [0u8; 20];
                    file.read_exact(&mut entry)?;
                    let cputype = u32::from_be_bytes(entry[0..4].try_into().unwrap());
                    let is_match = matches!(
                        (env::consts::ARCH, cputype),
                        ("aarch64", 0x0100_000C)      // CPU_TYPE_ARM64   = 0x0100000C
                            | ("x86_64", 0x0100_0007) // CPU_TYPE_X86_64  = 0x01000007
                    );
                    if is_match {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            // Single-arch 64-bit little-endian (aarch64 and x86_64).
            // cputype is at offset 4, stored as little-endian u32.
            [0xCF, 0xFA, 0xED, 0xFE] => {
                let mut ct = [0u8; 4];
                file.read_exact(&mut ct)?;
                let cputype = u32::from_le_bytes(ct);
                let is_match = matches!(
                    (env::consts::ARCH, cputype),
                    ("aarch64", 0x0100_000C)      // CPU_TYPE_ARM64   = 0x0100000C
                        | ("x86_64", 0x0100_0007) // CPU_TYPE_X86_64  = 0x01000007
                );
                Ok(is_match)
            }
            // Safe fallback for unsupported Mach-O formats.
            _ => Ok(false),
        }
    }

    // on Windows we call the dedicated variant of is_exec_by_magic_number.
    // TODO: here in case we ever port poof to Windows.
    #[cfg(target_os = "windows")]
    {
        return Ok(is_exec_by_magic_number(file_path));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Helper function to write a temporary file with the given bytes.
    fn write_tmp(bytes: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(bytes).unwrap();
        f.flush().unwrap();
        f
    }

    /// Build a minimal 20-byte ELF header for the current host architecture.
    #[cfg(target_os = "linux")]
    fn elf_header_for_current_arch() -> Vec<u8> {
        let arch = std::env::consts::ARCH;
        let (ei_data, e_machine): (u8, u16) = match arch {
            "x86_64" => (1, 0x3E),
            "aarch64" => (1, 0xB7),
            "x86" => (1, 0x03),
            "arm" => (1, 0x28),
            "riscv64" => (1, 0xF3),
            "powerpc64" => (1, 0x15),
            "s390x" => (2, 0x16),
            "loongarch64" => (1, 0x102),
            other => panic!("unsupported arch in test: {}", other),
        };
        let mut buf = vec![0u8; 20];
        // ELF magic
        buf[0] = 0x7F;
        buf[1] = 0x45;
        buf[2] = 0x4C;
        buf[3] = 0x46;
        // EI_CLASS = 64-bit (or 32-bit for x86/arm - doesn't matter for the check)
        buf[4] = 0x02;
        // EI_DATA
        buf[5] = ei_data;
        // EI_VERSION
        buf[6] = 0x01;
        // e_machine at offset 0x12
        let machine_bytes = if ei_data == 1 {
            e_machine.to_le_bytes()
        } else {
            e_machine.to_be_bytes()
        };
        buf[0x12] = machine_bytes[0];
        buf[0x13] = machine_bytes[1];
        buf
    }

    // *** is_exec_by_magic_number ********************************************

    #[test]
    fn test_is_exec_by_magic_number_shebang() {
        let f = write_tmp(&[0x23, 0x21, 0x2F, 0x62]); // "#!/b"
        assert!(is_exec_by_magic_number(f.path()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_is_exec_by_magic_number_elf() {
        let f = write_tmp(&ELF_MAGIC);
        assert!(is_exec_by_magic_number(f.path()));
    }

    #[test]
    fn test_is_exec_by_magic_number_non_exec_bytes() {
        let f = write_tmp(&[0x00, 0x01, 0x02, 0x03]);
        assert!(!is_exec_by_magic_number(f.path()));
    }

    #[test]
    fn test_is_exec_by_magic_number_nonexistent_path() {
        let path = std::path::Path::new("/tmp/poof_no_such_file_magic_test_abc123");
        assert!(!is_exec_by_magic_number(path));
    }

    // *** is_exec_for_current_arch *******************************************

    #[test]
    fn test_is_exec_for_current_arch_shebang() {
        let f = write_tmp(&[0x23, 0x21, 0x2F, 0x62, 0x69, 0x6E]); // "#!/bin"
        assert!(is_exec_for_current_arch(f.path()).unwrap());
    }

    #[test]
    fn test_is_exec_for_current_arch_non_exec_bytes() {
        let f = write_tmp(&[0x00, 0x01, 0x02, 0x03]);
        assert!(!is_exec_for_current_arch(f.path()).unwrap());
    }

    #[test]
    fn test_is_exec_for_current_arch_too_short() {
        // A file with fewer than 4 bytes causes read_exact to fail → Ok(false)
        let f = write_tmp(&[0x7F, 0x45]);
        assert!(!is_exec_for_current_arch(f.path()).unwrap());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_is_exec_for_current_arch_elf_matching_arch() {
        let header = elf_header_for_current_arch();
        let f = write_tmp(&header);
        assert!(is_exec_for_current_arch(f.path()).unwrap());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_is_exec_for_current_arch_elf_wrong_machine() {
        // Use a known-wrong machine type (EM_68K = 0x04) for all arches we support
        let mut header = elf_header_for_current_arch();
        // Overwrite e_machine with 0x0004 (EM_68K) in LE
        header[0x12] = 0x04;
        header[0x13] = 0x00;
        let f = write_tmp(&header);
        assert!(!is_exec_for_current_arch(f.path()).unwrap());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_is_exec_for_current_arch_elf_invalid_ei_data() {
        let mut header = elf_header_for_current_arch();
        // EI_DATA = 3 is not valid → function returns Ok(false)
        header[5] = 0x03;
        let f = write_tmp(&header);
        assert!(!is_exec_for_current_arch(f.path()).unwrap());
    }

    // *** macOS Mach-O helpers ***********************************************

    /// Build a minimal thin Mach-O (64-bit LE) buffer with the given cputype.
    #[cfg(target_os = "macos")]
    fn macho_thin(cputype: u32) -> Vec<u8> {
        let mut buf = vec![0u8; 20];
        buf[0..4].copy_from_slice(&[0xCF, 0xFA, 0xED, 0xFE]); // magic (64-bit LE)
        buf[4..8].copy_from_slice(&cputype.to_le_bytes()); // cputype (LE)
        buf
    }

    /// Build a minimal fat Mach-O buffer with the given cputypes.
    /// The fat header and each fat_arch entry use big-endian byte order.
    #[cfg(target_os = "macos")]
    fn macho_fat(cputypes: &[u32]) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]); // fat magic (BE)
        buf.extend_from_slice(&(cputypes.len() as u32).to_be_bytes()); // nfat_arch (BE)
        for &ct in cputypes {
            let mut entry = [0u8; 20]; // fat_arch is 5 × u32 = 20 bytes
            entry[0..4].copy_from_slice(&ct.to_be_bytes()); // cputype (BE)
            buf.extend_from_slice(&entry);
        }
        buf
    }

    /// Returns (current_arch_cputype, other_arch_cputype) for the two macOS
    /// architectures we currently support (aarch64 / x86_64).
    #[cfg(target_os = "macos")]
    fn macho_cputypes() -> (u32, u32) {
        const CPU_TYPE_ARM64: u32 = 0x0100_000C;
        const CPU_TYPE_X86_64: u32 = 0x0100_0007;
        match std::env::consts::ARCH {
            "aarch64" => (CPU_TYPE_ARM64, CPU_TYPE_X86_64),
            "x86_64" => (CPU_TYPE_X86_64, CPU_TYPE_ARM64),
            other => panic!("unsupported macOS arch in test: {}", other),
        }
    }

    // *** is_exec_by_magic_number (macOS) ************************************

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_exec_by_magic_number_macho() {
        // Any valid Mach-O magic (thin 64-bit LE) should be recognised.
        let f = write_tmp(&[0xCF, 0xFA, 0xED, 0xFE]);
        assert!(is_exec_by_magic_number(f.path()));
    }

    // *** is_exec_for_current_arch – thin Mach-O *****************************

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_exec_for_current_arch_macho_thin_current_arch() {
        let (current, _) = macho_cputypes();
        let f = write_tmp(&macho_thin(current));
        assert!(is_exec_for_current_arch(f.path()).unwrap());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_exec_for_current_arch_macho_thin_other_arch() {
        let (_, other) = macho_cputypes();
        let f = write_tmp(&macho_thin(other));
        assert!(!is_exec_for_current_arch(f.path()).unwrap());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_exec_for_current_arch_macho_thin_unknown_cputype() {
        let f = write_tmp(&macho_thin(0xDEAD_BEEF));
        assert!(!is_exec_for_current_arch(f.path()).unwrap());
    }

    // *** is_exec_for_current_arch – fat Mach-O ******************************

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_exec_for_current_arch_macho_fat_contains_current_arch() {
        let (current, _) = macho_cputypes();
        let f = write_tmp(&macho_fat(&[current]));
        assert!(is_exec_for_current_arch(f.path()).unwrap());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_exec_for_current_arch_macho_fat_both_archs() {
        let (current, other) = macho_cputypes();
        // Order shouldn't matter; the function iterates all entries.
        let f = write_tmp(&macho_fat(&[other, current]));
        assert!(is_exec_for_current_arch(f.path()).unwrap());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_exec_for_current_arch_macho_fat_only_other_arch() {
        let (_, other) = macho_cputypes();
        let f = write_tmp(&macho_fat(&[other]));
        assert!(!is_exec_for_current_arch(f.path()).unwrap());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_exec_for_current_arch_macho_fat_zero_archs() {
        // nfat_arch = 0 means no entries to iterate → Ok(false).
        let f = write_tmp(&macho_fat(&[]));
        assert!(!is_exec_for_current_arch(f.path()).unwrap());
    }
}
