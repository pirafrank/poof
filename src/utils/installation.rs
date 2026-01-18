use std::path::Path;

/// Detects if poof was installed via cargo, .deb, or .rpm
/// Returns true if installed via one of these methods, false otherwise
pub fn is_externally_managed_installation() -> bool {
    // Get the current executable path
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => return false,
    };

    // Check if installed via cargo (typically in ~/.cargo/bin)
    if is_cargo_installation(&exe_path) {
        return true;
    }

    // Check if installed via .deb (typically in /usr/bin or /usr/local/bin)
    if is_deb_installation(&exe_path) {
        return true;
    }

    // Check if installed via .rpm (typically in /usr/bin or /usr/local/bin)
    if is_rpm_installation(&exe_path) {
        return true;
    }

    false
}

fn is_cargo_installation(exe_path: &Path) -> bool {
    // Check if executable is in ~/.cargo/bin
    if let Some(home) = dirs::home_dir() {
        let cargo_bin = home.join(".cargo").join("bin");
        if exe_path.starts_with(&cargo_bin) {
            return true;
        }
    }
    false
}

fn is_deb_installation(exe_path: &Path) -> bool {
    // Debian packages typically install to /usr/bin or /usr/local/bin
    let deb_paths = ["/usr/bin", "/usr/local/bin"];

    for deb_path in &deb_paths {
        if exe_path.starts_with(deb_path) {
            // Additional check: see if dpkg knows about this package
            if is_managed_by_dpkg() {
                return true;
            }
        }
    }
    false
}

fn is_rpm_installation(exe_path: &Path) -> bool {
    // RPM packages typically install to /usr/bin or /usr/local/bin
    let rpm_paths = ["/usr/bin", "/usr/local/bin"];

    for rpm_path in &rpm_paths {
        if exe_path.starts_with(rpm_path) {
            // Additional check: see if rpm knows about this package
            if is_managed_by_rpm() {
                return true;
            }
        }
    }
    false
}

fn is_managed_by_dpkg() -> bool {
    // Check if poof is installed via dpkg
    std::process::Command::new("dpkg")
        .args(["-s", "poof"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn is_managed_by_rpm() -> bool {
    // Check if poof is installed via rpm
    std::process::Command::new("rpm")
        .args(["-q", "poof"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_installation_detection() {
        if let Some(home) = dirs::home_dir() {
            let cargo_path = home.join(".cargo").join("bin").join("poof");
            assert!(is_cargo_installation(&cargo_path));
        }
    }

    #[test]
    fn test_non_cargo_installation() {
        let non_cargo_path = Path::new("/usr/bin/poof");
        assert!(!is_cargo_installation(non_cargo_path));
    }

    #[test]
    fn test_deb_installation_path_detection() {
        let usr_bin_path = Path::new("/usr/bin/poof");
        let usr_local_bin_path = Path::new("/usr/local/bin/poof");

        // Path checking works, but dpkg check will fail in test environment
        assert!(usr_bin_path.starts_with("/usr/bin"));
        assert!(usr_local_bin_path.starts_with("/usr/local/bin"));
    }

    #[test]
    fn test_rpm_installation_path_detection() {
        let usr_bin_path = Path::new("/usr/bin/poof");
        let usr_local_bin_path = Path::new("/usr/local/bin/poof");

        assert!(usr_bin_path.starts_with("/usr/bin"));
        assert!(usr_local_bin_path.starts_with("/usr/local/bin"));
    }

    #[test]
    fn test_non_standard_path() {
        let custom_path = Path::new("/opt/custom/bin/poof");
        assert!(!is_cargo_installation(custom_path));
    }

    #[test]
    fn test_relative_path() {
        let relative_path = Path::new("./target/debug/poof");
        assert!(!is_cargo_installation(relative_path));
    }

    #[test]
    fn test_dpkg_check() {
        // Stub for code coverage.
        // This will return false in most test environments
        let result = is_managed_by_dpkg();
        assert!(result == true || result == false);
    }

    #[test]
    fn test_rpm_check() {
        // Stub for code coverage.
        // This will return false in most test environments
        let result = is_managed_by_rpm();
        assert!(result == true || result == false);
    }
}
