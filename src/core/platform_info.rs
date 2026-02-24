//! Runtime platform inspection helpers.
//!
//! Provides functions for querying the OS version, architecture endianness,
//! shell environment, PATH contents, and build-time version information.

use crate::constants::*;
use crate::utils::string;

#[cfg(not(target_os = "windows"))]
const ENV_PATH_SEPARATOR: &str = ":";
#[cfg(target_os = "windows")]
const ENV_PATH_SEPARATOR: &str = ";";

/// Return the separator used in the PATH environment variable.
pub fn env_path_separator() -> &'static str {
    ENV_PATH_SEPARATOR
}

/// Returns a static string containing the version information.
/// It uses Box::leak to convert a String into a &'static str.
/// This is a workaround to avoid using a global static variable.
pub fn long_version() -> &'static str {
    #[cfg(static_linking)]
    let linking_type = "statically linked";
    #[cfg(dynamic_linking)]
    let linking_type = "dynamically linked";
    Box::leak(
        format!(
            "Version   : {}\nCommit    : {}\nBuild Date: {}\nBuilt with: {} ({}){}\n{}",
            VERSION,
            COMMIT,
            BUILD_DATE,
            COMPILE_C_LIB,
            linking_type,
            get_glibc_version_string(),
            release_url()
        )
        .into_boxed_str(),
    )
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
/// Returns the Glibc version if the build is GNU, otherwise returns an empty string.
pub fn get_glibc_version() -> Option<String> {
    let result = std::panic::catch_unwind(|| unsafe {
        let version_ptr = libc::gnu_get_libc_version();
        if !version_ptr.is_null() {
            let version_cstr = std::ffi::CStr::from_ptr(version_ptr);
            return version_cstr.to_str().ok().map(|s| s.to_string());
        }
        None
    });
    result.unwrap_or(None)
}

#[cfg(not(all(target_os = "linux", target_env = "gnu")))]
pub fn get_glibc_version() -> Option<String> {
    None
}

/// Returns the GitHub releases URL for the current version of poof.
pub fn release_url() -> String {
    format!("{}/releases/tag/v{}", THIS_REPO_URL, VERSION)
}

/// Returns the Glibc string to show in platform info method
fn get_glibc_version_string() -> String {
    match get_glibc_version() {
        Some(version) => {
            format!("\nRunning on: glibc v{}", version)
        }
        None => "".to_string(),
    }
}

/// Returns the short description string of the application.
pub fn short_description() -> &'static str {
    DESCRIPTION
}

/// Read an environment variable, returning [`UNKNOWN`] if it is not set.
pub fn get_env_var(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| UNKNOWN.to_string())
}

/// Return a human-readable string describing the OS name and version.
///
/// On Linux, attempts `lsb_release -ds` or `/etc/os-release`. On macOS uses
/// `sw_vers`. On Windows uses `cmd /c ver`. Returns [`UNKNOWN`] on failure.
pub fn get_os_version() -> String {
    if cfg!(target_os = "linux") {
        // Try to detect Linux distribution and version
        std::process::Command::new("sh")
            .arg("-c")
            .arg("(lsb_release -ds 2>/dev/null) || (cat /etc/os-release | grep PRETTY_NAME | cut -d '=' -f 2 | tr -d '\"')")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| UNKNOWN.to_string())
    } else if cfg!(target_os = "macos") {
        // Get macOS version
        std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .map(|o| format!("macOS {}", String::from_utf8_lossy(&o.stdout).trim()))
            .unwrap_or_else(|_| UNKNOWN.to_string())
    } else if cfg!(target_os = "windows") {
        // Get Windows version
        std::process::Command::new("cmd")
            .args(["/c", "ver"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| UNKNOWN.to_string())
    } else {
        UNKNOWN.to_string()
    }
}

/// Return a string describing the platform byte order (`"Little Endian"`, `"Big Endian"`, or `"Unknown Endian"`).
#[cfg(target_endian = "little")]
pub fn get_platform_endianness() -> String {
    "Little Endian".to_string()
}

#[cfg(target_endian = "big")]
pub fn get_platform_endianness() -> String {
    "Big Endian".to_string()
}

#[cfg(all(not(target_endian = "little"), not(target_endian = "big")))]
pub fn get_platform_endianness() -> String {
    "Unknown Endian".to_string()
}

/// Return a string containing the current shell name and its reported version.
///
/// Reads the `SHELL` environment variable and invokes the shell binary with
/// `--version` to obtain its version string. Returns [`UNKNOWN`] when the
/// shell cannot be determined or the version query fails.
pub fn get_shell_info() -> String {
    let shell_name = get_env_var("SHELL");
    let shell_version = if shell_name != UNKNOWN {
        std::process::Command::new(&shell_name)
            .arg("--version")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| UNKNOWN.to_string())
    } else {
        UNKNOWN.to_string()
    };
    format!("{} version: {}", shell_name, shell_version)
}

/// Return the zero-based position of `dir` in the `PATH` environment variable.
///
/// Returns `-1` when `dir` is not present in `PATH`.
pub fn check_dir_in_path(dir: &str) -> i16 {
    let path = get_env_var("PATH");
    let sep = env_path_separator();
    string::position_of_str_in_string(path, sep, dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_env_path_separator_unix() {
        #[cfg(not(target_os = "windows"))]
        {
            assert_eq!(env_path_separator(), ":");
        }
    }

    #[test]
    fn test_env_path_separator_windows() {
        #[cfg(target_os = "windows")]
        {
            assert_eq!(env_path_separator(), ";");
        }
    }

    #[test]
    fn test_long_version_format() {
        let version = long_version();
        // Check that the version string contains expected keywords
        assert!(version.contains("Version"));
        assert!(version.contains("Commit"));
        assert!(version.contains("Build Date"));
        assert!(version.contains("Built with"));
        // Check for linking type
        assert!(
            version.contains("statically linked") || version.contains("dynamically linked"),
            "Version string should contain linking type"
        );
    }

    #[test]
    fn test_short_description() {
        let desc = short_description();
        assert_eq!(desc, DESCRIPTION);
        assert!(!desc.is_empty(), "Description should not be empty");
    }

    #[test]
    fn test_get_env_var_existing() {
        // Test with PATH which should always exist
        let path = get_env_var("PATH");
        assert_ne!(path, UNKNOWN, "PATH environment variable should exist");
        assert!(!path.is_empty(), "PATH should not be empty");
    }

    #[test]
    fn test_get_env_var_nonexistent() {
        // Test with a variable that definitely doesn't exist
        let result = get_env_var("NONEXISTENT_VAR_THAT_SHOULD_NOT_EXIST_12345");
        assert_eq!(result, UNKNOWN);
    }

    #[test]
    fn test_get_os_version_not_empty() {
        let os_version = get_os_version();
        // Should return something, not empty
        assert!(!os_version.is_empty(), "OS version should not be empty");
    }

    #[test]
    fn test_get_platform_endianness() {
        let endianness = get_platform_endianness();
        // Should be one of the known values
        assert!(
            endianness == "Little Endian"
                || endianness == "Big Endian"
                || endianness == "Unknown Endian",
            "Endianness should be Little, Big, or Unknown"
        );
        // Most modern systems are little endian
        #[cfg(target_endian = "little")]
        {
            assert_eq!(endianness, "Little Endian");
        }
        #[cfg(target_endian = "big")]
        {
            assert_eq!(endianness, "Big Endian");
        }
    }

    #[test]
    fn test_get_shell_info_format() {
        let shell_info = get_shell_info();
        // Should contain "version:" keyword
        assert!(
            shell_info.contains("version:"),
            "Shell info should contain 'version:'"
        );
        assert!(!shell_info.is_empty(), "Shell info should not be empty");
    }

    #[test]
    fn test_check_dir_in_path_first_entry() {
        // Get the PATH and check for its first entry
        let path = get_env_var("PATH");
        if path != UNKNOWN {
            let sep = env_path_separator();
            let parts: Vec<&str> = path.split(sep).collect();
            if !parts.is_empty() {
                let first_dir = parts[0];
                let position = check_dir_in_path(first_dir);
                assert_eq!(position, 0, "First directory should be at position 0");
            }
        }
    }

    #[test]
    fn test_check_dir_in_path_not_found() {
        let position = check_dir_in_path("/nonexistent/directory/that/should/not/be/in/path/12345");
        assert_eq!(position, -1, "Nonexistent directory should return -1");
    }

    #[test]
    fn test_check_dir_in_path_existing() {
        // Most Unix-like systems have /usr/bin in PATH
        #[cfg(not(target_os = "windows"))]
        {
            let position = check_dir_in_path("/usr/bin");
            // Should either be found (>= 0) or not found (-1)
            // We can't guarantee it's in PATH, but we can test the function works
            assert!(
                position >= -1,
                "Position should be valid (>= -1), got {}",
                position
            );
        }
    }

    #[test]
    fn test_check_dir_in_path_multiple_entries() {
        use temp_env::with_var;

        // Build a fake PATH-like string and test
        let temp_path = format!(
            "/first/path{}/second/path{}/third/path",
            env_path_separator(),
            env_path_separator()
        );

        // Set our test PATH - temp-env automatically restores it after the closure
        with_var("PATH", Some(&temp_path), || {
            assert_eq!(check_dir_in_path("/first/path"), 0);
            assert_eq!(check_dir_in_path("/second/path"), 1);
            assert_eq!(check_dir_in_path("/third/path"), 2);
            assert_eq!(check_dir_in_path("/nonexistent"), -1);
        });
    }

    #[test]
    #[serial]
    fn test_env_path_separator_consistency() {
        // Ensure the separator matches the platform
        let sep = env_path_separator();
        assert!(!sep.is_empty(), "Separator should not be empty");
        assert!(
            sep == ":" || sep == ";",
            "Separator should be ':' or ';', got '{}'",
            sep
        );
    }
}
