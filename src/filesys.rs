use log::{debug, warn};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use crate::selector::SUPPORTED_EXTENSIONS;
use crate::utils;

// Constants for magic numbers
#[cfg(target_os = "macos")]
const MACHO_MAGIC_NUMBERS: &[[u8; 4]] = &[
    [0xFE, 0xED, 0xFA, 0xCE], // Mach-O 32-bit (little-endian)
    [0xFE, 0xED, 0xFA, 0xCF], // Mach-O 64-bit (little-endian)
    [0xCE, 0xFA, 0xED, 0xFE], // Mach-O 32-bit (big-endian)
    [0xCF, 0xFA, 0xED, 0xFE], // Mach-O 64-bit (big-endian)
    [0xCA, 0xFE, 0xBA, 0xBE], // Mach-O universal ('fat') binary (little-endian)
    [0xBE, 0xBA, 0xFE, 0xCA], // Mach-O universal ('fat') binary (big-endian)
];

#[cfg(target_os = "linux")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    // Linux expects ELF binaries
    buffer == &[0x7F, 0x45, 0x4C, 0x46] // ELF
}

#[cfg(target_os = "windows")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    // Windows expects PE binaries (MZ header).
    // Checking only the first two bytes because the other two may change,
    // as they depend on the DOS stub.
    buffer[..2] == [0x4D, 0x5A]
}

#[cfg(target_os = "macos")]
fn is_exec_magic(buffer: &[u8; 4]) -> bool {
    // macOS expects Mach-O formats
    MACHO_MAGIC_NUMBERS.contains(buffer)
}

#[cfg(not(target_os = "windows"))]
fn is_exec_by_magic_number(path: &PathBuf) -> bool {
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0u8; 4];
        if file.read_exact(&mut buffer).is_ok() {
            return is_exec_magic(&buffer);
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn is_exec_by_magic_number(path: &PathBuf) -> bool {
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

pub fn find_exec_files_in_dir(dir: &PathBuf) -> Vec<PathBuf> {
    let mut result: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                // check criteria to determine if a file is a binary
                // 1. Check if the file is a regular file
                // 2. Check if the file is an executable by checking the magic number
                if file_type.is_file() && is_exec_by_magic_number(&entry.path()) {
                    result.push(entry.path());
                }
            }
        }
    }
    result
}

fn strip_supported_extensions(path: &Path) -> &str {
    let filename = utils::get_file_name(path);
    SUPPORTED_EXTENSIONS
        .iter()
        .find_map(|ext| filename.strip_suffix(ext))
        .unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(filename)
        })
}

pub fn find_exec_files_from_extracted_archive(archive_path: &Path) -> Vec<PathBuf> {
    let archive_parent = archive_path.parent().unwrap();
    // Get the filename without the extension
    // and create the path of a directory with the same name as the archive, minus the extension.
    // If it exists, we will search for executables in that directory.
    // If it doesn't exist, we will search for executables in the parent directory.
    // This is useful for archives that contain a directory with the same name as the archive.
    let filename_no_ext_str = strip_supported_extensions(archive_path);
    let dir = archive_parent.join(filename_no_ext_str);
    if dir.exists() {
        find_exec_files_in_dir(&dir)
    } else {
        find_exec_files_in_dir(&PathBuf::from(archive_parent))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn is_executable(path: &PathBuf) -> bool {
    // Check if the file is executable
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.is_file() {
            let permissions = metadata.permissions();
            return permissions.mode() & 0o111 != 0;
        }
    }
    false
}

#[cfg(not(target_os = "windows"))]
pub fn make_executable(file: &Path) {
    if !file.is_file() {
        debug!("File {} is not a regular file", file.to_string_lossy());
        return;
    }
    debug!("Making {} executable", file.to_string_lossy());
    // Unix-like systems require setting executable permissions
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(file).unwrap().permissions();
    // Add executable bits to current permissions (equivalent to chmod +x)
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(file, perms).unwrap();
    debug!("Set executable permissions for {}", file.display());
}

pub fn copy_file(source: &PathBuf, target: &PathBuf) -> Result<(), String> {
    debug!(
        "Copying file from {} to {}",
        source.display(),
        target.display()
    );
    if let Err(e) = std::fs::copy(source, target) {
        let e_msg = format!(
            "Error copying {} to {}: {}",
            source.display(),
            target.display(),
            e
        );
        return Err(e_msg);
    };
    debug!("File copied successfully");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn create_symlink(
    source: &PathBuf,
    target: &PathBuf,
    remove_existing: bool,
) -> Result<(), String> {
    use log::info;

    let msg = if remove_existing { "" } else { " NOT" };
    debug!(
        "Creating symlink {} -> {},{} removing existing",
        source.display(),
        target.display(),
        msg
    );
    if target.exists() {
        if remove_existing {
            if let Err(e) = std::fs::remove_file(target) {
                return Err(format!("Cannot remove {}, Error: {}", target.display(), e));
            }
            debug!("Removed existing symlink {}", target.display());
        } else {
            // If the symlink already exists and we don't want to remove it, skip.
            warn!("Symlink {} already exists. Skipping.", target.display());
            return Ok(());
        }
    }

    // Create a symlink in the target directory pointing to the installed binary.
    match std::os::unix::fs::symlink(source, target) {
        Ok(_) => {
            info!(
                "Symlink created: {} -> {}",
                source.display(),
                target.display()
            );
        }
        Err(e) => {
            let e_msg = format!(
                "Error creating symlink {} -> {}: {}",
                source.display(),
                target.display(),
                e
            );
            return Err(e_msg);
        }
    }
    Ok(())
}
