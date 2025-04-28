// Constants for version information
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = "magic manager of pre-built software";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const COMMIT: &str = env!("GIT_COMMIT_HASH");
const BUILD_DATE: &str = env!("BUILD_DATE");

// Constants for platform information
const UNKNOWN: &str = "Unknown";

/// Returns a static string containing the version information.
/// It uses Box::leak to convert a String into a &'static str.
/// This is a workaround to avoid using a global static variable.
pub fn long_version() -> &'static str {
    Box::leak(
        format!(
            "\nVersion: {}\nCommit: {}\nBuild Date: {}",
            VERSION, COMMIT, BUILD_DATE
        )
        .into_boxed_str(),
    )
}

pub fn short_description() -> &'static str {
    DESCRIPTION
}

fn get_env_var(var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| UNKNOWN.to_string())
}

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

pub fn get_platform_endianness() -> String {
    (if cfg!(target_endian = "little") {
        "Little Endian"
    } else if cfg!(target_endian = "big") {
        "Big Endian"
    } else {
        "Unknown Endian"
    })
    .to_string()
}

fn get_shell_info() -> String {
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

/// Print platform information useful for debug purposes.
pub fn debug_info() {
    print!("\n{} - {}\n{}\n", APP_NAME, DESCRIPTION, long_version());
    // Print system information
    println!("\nPlatform Information:");
    println!("  OS family: {}", std::env::consts::OS);
    println!("  OS version: {}", get_os_version());
    println!("  Architecture: {}", std::env::consts::ARCH);
    println!("  Endianness: {}", get_platform_endianness());
    println!("  Family: {}", std::env::consts::FAMILY);
    println!(
        "  Kernel: {}",
        std::process::Command::new("uname")
            .arg("-a")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| UNKNOWN.to_string())
    );
    println!(
        "  Executable: {}",
        std::env::current_exe().unwrap_or_default().display()
    );
    println!(
        "  Current directory: {}",
        std::env::current_dir().unwrap_or_default().display()
    );

    // Environment variables
    print!("\nEnvironment:\n");
    println!("  SHELL: {}", get_shell_info());
    println!("  USER: {}", get_env_var("USER"));
    println!("  HOME: {}", get_env_var("HOME"));
    println!("  PATH: {}", get_env_var("PATH"));
}
