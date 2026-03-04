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
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("definitely_missing_exec_magic_test");
    assert!(!path.exists());
    assert!(!is_exec_by_magic_number(&path));
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
