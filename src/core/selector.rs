use crate::models::asset_triple::AssetTriple;
use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::constants::SUPPORTED_EXTENSIONS;

lazy_static! {
    static ref OPERATING_SYSTEM: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("windows", vec!["windows", "win"]);
        m.insert("macos", vec!["macos", "darwin", "mac", "osx"]);
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
        m.insert("x86", vec!["x86", "386", "586", "686", "32-bit"]);
        m.insert("x86_64", vec!["x86_64", "x64", "amd64"]);
        // order matters here, from more specific to less specific
        // arm assets will run on any armv7 device the armv7 poof build target runs on.
        m.insert("armv7", vec!["armv7l", "armhf", "armv7", "arm"]);
        m.insert("aarch64", vec!["aarch64", "arm64"]);
        // powerpc64le support
        m.insert("powerpc", vec!["powerpcle", "ppcle"]);
        m.insert("powerpc64", vec!["powerpc64le", "ppc64le"]);
        // note: de-facto are all riscv64 are riscv64gc if they run can Linux,
        // as linux needs the gc extensions.
        m.insert("riscv64", vec!["riscv64gc", "riscv64"]);
        // s390x 64bit support
        m.insert("s390x", vec!["s390x"]);
        // loongarch64 support
        m.insert("loongarch64", vec!["loongarch64"]);
        m
    };
}

#[cfg(target_arch = "arm")]
pub fn hf_supported() -> &'static bool {
    #[cfg(target_feature = "vfp2")]
    {
        &true
    }

    #[cfg(not(target_feature = "vfp2"))]
    {
        &false
    }
}

fn is_exec_name_only(arch: &&str, s: &str) -> bool {
    // get index of arch in string
    let arch_index = s.find(arch);
    if let Some(index) = arch_index {
        // check if the string after the arch is empty
        let after_arch = &s[index + arch.len()..];
        if after_arch.is_empty() {
            return true;
        }
    }
    false
}

/// Returns true if the input string has patterns compatible with the current environment.
pub fn is_env_compatible(input: &str) -> bool {
    // get the current environment as an AssetTriple
    let t = AssetTriple::default();
    is_triple_compatible(input, &t)
}

/// Returns true if the input string has patterns compatible with the given OS, ARCH, triple.
fn is_triple_compatible(input: &str, t: &AssetTriple) -> bool {
    // Convert item to lowercase for comparison as
    // OPERATING_SYSTEM and CPU_ARCH are lowercase in the code above.
    let item = input.to_lowercase();

    // OPERATING_SYSTEM
    // Check if this OS matches our current OS
    if !OPERATING_SYSTEM
        .get(t.get_os().as_str())
        .is_some_and(|aliases| aliases.iter().any(|alias| item.contains(alias)))
    {
        return false;
    }

    // CPU_ARCH
    // Check if this architecture matches our current architecture
    let matching_arch_alias = match CPU_ARCH.get(t.get_arch().as_str()) {
        Some(aliases) => {
            let found = aliases.iter().find(|&&alias| item.contains(alias));
            if found.is_none() {
                return false;
            }
            found.unwrap()
        }
        None => return false,
    };
    if is_exec_name_only(matching_arch_alias, &item) {
        return true;
    } // else continue execution

    // SUPPORTED_EXTENSIONS
    // Check if the file extension is supported
    if !SUPPORTED_EXTENSIONS
        .iter()
        .any(|&format| item.ends_with(format))
    {
        return false;
    }

    // MUSL
    // Check if the binary is musl
    if t.is_musl() && !item.contains("musl") {
        return false;
    }

    // if we got this far, we have a winner
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_env_compatible() {
        let linux = String::from("ipinfo_3.3.1_linux_amd64.tar.gz");
        let windows = String::from("ipinfo_3.3.1_windows_amd64.zip");

        if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            assert_eq!(true, is_env_compatible(&linux));
            assert_eq!(false, is_env_compatible(&windows));
        }
        // If running on Windows with MSVC, this should pass.
        if cfg!(all(
            target_os = "windows",
            target_arch = "x86_64",
            target_env = "msvc"
        )) {
            assert_eq!(true, is_env_compatible(&windows));
            assert_eq!(false, is_env_compatible(&linux));
        }
    }
}
