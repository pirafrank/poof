use std::collections::HashMap;
use std::env::consts::{FAMILY, ARCH, OS};
use lazy_static::lazy_static;

const SUPPORTED_FORMATS: [&str; 2] = ["tar.gz", "zip"];

lazy_static! {
    static ref OPERATING_SYSTEM: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("windows", vec!["windows", "win"]);
        m.insert("macos", vec!["macos", "darwin", "mac"]);
        m.insert("linux", vec!["linux"]);
        m.insert("openbsd", vec!["openbsd"]);
        m.insert("freebsd", vec!["freebsd"]);
        m.insert("netbsd", vec!["netbsd"]);
        m
    };
}

lazy_static! {
    static ref CPU_ARCH: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("x86", vec!["x86", "386", "686", "32-bit"]);
        m.insert("x86_64", vec!["x86_64", "x64", "amd64"]);
        m.insert("armv5", vec!["armv5",]);
        m.insert("armv6", vec!["armv6",]);
        m.insert("arm", vec!["arm", "armv7"]);
        m.insert("aarch64", vec!["aarch64","arm64",]);

        if cfg!(target_endian = "big") {
          m.insert("mips", vec!["mips", "mips32"]);
          m.insert("mips64", vec!["mips64",]);
          m.insert("powerpc", vec!["ppc",]);
          m.insert("powerpc64", vec!["ppc64",]);
        } else {
          m.insert("mips", vec!["mipsle", "mips32le"]);
          m.insert("mips64", vec!["mips64le",]);
          m.insert("powerpc", vec!["ppcle",]);
          m.insert("powerpc64", vec!["ppc64le",]);
        }

        m.insert("riscv32", vec!["riscv32", "riscv"]);
        m.insert("riscv64", vec!["riscv64",]);
        m.insert("s390x", vec!["s390x",]);
        m
    };
}

pub fn get_platform_info() -> String {
    let arch = ARCH;
    let os = OS;
    format!("Platform: {}, OS: {}, Architecture: {}", FAMILY, os, arch)
}

#[cfg(target_arch = "arm")]
pub fn detect_fpu() -> &'static str {
  #[cfg(target_feature = "vfp2")]
  {
    "HF Supported"
  }

  #[cfg(not(target_feature = "vfp2"))]
  {
    "HF Not Supported"
  }
}

// Change the function signature to accept Vec<String>
pub fn get_exact(input: Vec<String>) -> String {
  // Iterate through inputs and find a match for current OS and architecture
  for item_str in input {
    // Convert item to lowercase for comparison as
    // OPERATING_SYSTEM and CPU_ARCH are lowercase in the code above.
    let item = item_str.to_lowercase();

    // patch: avoid musl compiled binaries on linux
    if item.contains("musl") && OS == "linux" {
      continue;
    }

    // OPERATING_SYSTEM
    // Check if this OS matches our current OS
    if !OPERATING_SYSTEM.get(OS)
      .map_or(false, |aliases| aliases.iter().any(|alias| item.contains(alias))) {
      continue;
    }

    // CPU_ARCH
    // Check if this architecture matches our current architecture
    if !CPU_ARCH.get(ARCH)
      .map_or(false, |aliases| aliases.iter().any(|alias| item.contains(alias))) {
      continue;
    }

    // SUPPORTED_FORMATS
    // Check if the file extension is supported
    if !SUPPORTED_FORMATS.iter().any(|&format| item.ends_with(format)) {
      continue;
    }

    // if we have a winner, return the original item string
    return item_str;
  }

  // Default return if no match is found
  String::from("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_info() {
        let info = get_platform_info();
        println!("{}", info);
        assert!(info.contains("Platform:"));
        assert!(info.contains("OS:"));
        assert!(info.contains("Architecture:"));
    }

    #[test]
    fn test_get_exact() {
      // Update the input to be Vec<String>
      let input = vec![
        "deb.sh".to_string(),
        "ipinfo_3.3.1_darwin_amd64.tar.gz".to_string(),
        "ipinfo_3.3.1_darwin_arm64.tar.gz".to_string(),
        "ipinfo_3.3.1_dragonfly_amd64.tar.gz".to_string(),
        "ipinfo_3.3.1_freebsd_386.tar.gz".to_string(),
        "ipinfo_3.3.1_freebsd_amd64.tar.gz".to_string(),
        "ipinfo_3.3.1_freebsd_arm.tar.gz".to_string(),
        "ipinfo_3.3.1_freebsd_arm64.tar.gz".to_string(),
        "ipinfo_3.3.1_linux_386.deb".to_string(),
        "ipinfo_3.3.1_linux_386.tar.gz".to_string(),
        "ipinfo_3.3.1_linux_amd64.deb".to_string(),
        "ipinfo_3.3.1_linux_amd64.tar.gz".to_string(),
        "ipinfo_3.3.1_linux_arm.deb".to_string(),
        "ipinfo_3.3.1_linux_arm.tar.gz".to_string(),
        "ipinfo_3.3.1_linux_arm64.deb".to_string(),
        "ipinfo_3.3.1_linux_arm64.tar.gz".to_string(),
        "ipinfo_3.3.1_netbsd_386.tar.gz".to_string(),
        "ipinfo_3.3.1_netbsd_amd64.tar.gz".to_string(),
        "ipinfo_3.3.1_netbsd_arm.tar.gz".to_string(),
        "ipinfo_3.3.1_netbsd_arm64.tar.gz".to_string(),
        "ipinfo_3.3.1_openbsd_386.tar.gz".to_string(),
        "ipinfo_3.3.1_openbsd_amd64.tar.gz".to_string(),
        "ipinfo_3.3.1_openbsd_arm.tar.gz".to_string(),
        "ipinfo_3.3.1_openbsd_arm64.tar.gz".to_string(),
        "ipinfo_3.3.1_solaris_amd64.tar.gz".to_string(),
        "ipinfo_3.3.1_windows_386.zip".to_string(),
        "ipinfo_3.3.1_windows_amd64.zip".to_string(),
        "ipinfo_3.3.1_windows_arm.zip".to_string(),
        "ipinfo_3.3.1_windows_arm64.zip".to_string(),
        "macos.sh".to_string(),
        "windows.ps1".to_string(),
      ];
      let result = get_exact(input);

      // Warning: This assertion depends on the platform running the test.
      // If running on Linux AMD64, this should pass.
      if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
          assert_eq!(result, "ipinfo_3.3.1_linux_amd64.tar.gz");
      }
      // If running on Windows with MSVC, this should pass.
      if cfg!(all(target_os = "windows", target_arch = "x86_64", target_env = "msvc")) {
        assert_eq!(result, "ipinfo_3.3.1_windows_amd64.zip");
      }
    }
}
