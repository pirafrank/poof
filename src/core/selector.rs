use crate::{files::utils::contains_alias_token, models::asset_triple::AssetTriple};
use lazy_static::lazy_static;
use std::{cmp::max, collections::HashMap};

use crate::constants::SUPPORTED_EXTENSIONS;

lazy_static! {
    static ref OPERATING_SYSTEM: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        //m.insert("windows", vec!["windows", "win"]);
        m.insert("macos", vec!["macos", "darwin", "osx", "mac"]);
        m.insert("linux", vec!["linux"]);
        //m.insert("openbsd", vec!["openbsd"]);
        //m.insert("freebsd", vec!["freebsd"]);
        //m.insert("netbsd", vec!["netbsd"]);
        m
    };
}

lazy_static! {
    static ref CPU_ARCH: HashMap<&'static str, Vec<&'static str>> = {
        let mut m = HashMap::new();
        m.insert("x86", vec!["x86", "i386", "i586", "i686", "386", "586", "686", "32-bit"]);
        m.insert("x86_64", vec!["x86_64", "x86-64", "x64", "amd64"]);
        // order matters here, from more specific to less specific
        // arm assets will run on any armv7 device the armv7 poof build target runs on.
        // armhf is a more specific alias for armv7, so it should be listed before armv7,
        // yet it may be some armv6 asset, which armv7 would run anyway, so we list it
        // but after armv7 to let it have less priority than armv7.
        m.insert("armv7", vec!["armv7l", "armv7", "armhf", "armv6", "arm"]);
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

/// Returns `true` if `item` has what looks like a real file extension (non-empty, ≤4 chars, not all digits).
fn has_extension(item: &str) -> bool {
    // going case insensitive to avoid false positives for AppImage assets
    let item = item.to_lowercase();
    // if the item does not contain a dot, it does not have an extension,
    if !item.contains(".") {
        return false;
    }
    // we split to try to figure out an extension whatsoever
    let p: Vec<&str> = item.split(".").collect();
    // if empty, no extension. return false.
    if p.len() <= 1 {
        return false;
    }
    // we have one. is it real?
    let last = p.last().unwrap();
    // if empty, no extension. return false.
    if last.is_empty() {
        return false;
    }
    // hotfix to avoid false positives for AppImage assets
    if last == &"appimage" {
        return true;
    }
    // if too long, unlikely to be a real extension. return false.
    if last.len() > 9 {
        return false;
    }
    // must be alphanumeric (a-z, 0-9) but not purely numeric,
    // otherwise it's likely a false positive.
    if !last.chars().all(|c| c.is_ascii_alphanumeric()) || last.chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }
    // if we got this far, let's assume it's a real one.
    true
}

/// Returns the most compatible assets from the given list of assets
pub fn get_env_compatible_assets<T, F>(assets: &[T], extractor_fn: F) -> Option<Vec<T>>
where
    T: Clone,
    F: Fn(&T) -> &str,
{
    let t = AssetTriple::default();
    get_triple_compatible_assets(assets, &t, &extractor_fn)
}

/// Returns the most compatible asset from the given list of assets
///
/// # Arguments
///
/// * `assets` - The list of assets to select from.
/// * `t` - The asset triple to use for scoring.
/// * `extractor_fn` - The function to extract the asset name from the asset.
///
/// # Returns
///
/// A vector of the most compatible assets.
/// If no compatible assets are found, returns `None`.
pub fn get_triple_compatible_assets<T, F>(
    assets: &[T],
    t: &AssetTriple,
    extractor_fn: F,
) -> Option<Vec<T>>
where
    T: Clone,
    F: Fn(&T) -> &str,
{
    let mut map: HashMap<i32, Vec<T>> = HashMap::new();
    let mut max_score: i32 = 0;
    for asset in assets {
        let score = get_triple_score(extractor_fn(asset), t);
        if score > 0 {
            max_score = max(max_score, score);
            map.entry(score).or_default().push(asset.clone());
        }
    }
    if max_score > 0 {
        map.get(&max_score).cloned()
    } else {
        None
    }
}

/// Get score for the input string
fn get_triple_score(input: &str, t: &AssetTriple) -> i32 {
    // Initialize score to 0
    let mut score: i32 = 0;

    // Convert item to lowercase for comparison as
    // OPERATING_SYSTEM and CPU_ARCH are lowercase in the code above.
    let item = input.to_lowercase();

    // AVOID CHECKSUMS AND SIGNATURE FILES
    // Avoid checksum files and signature files as false positive binary assets.
    // we discard it by returning -1 as deal-breaker.
    if is_checksum_file(&item) || is_signature_file(&item) {
        return -1;
    }

    // current_os is the operating system from the AssetTriple.
    // AssetTriple defaults to the operating system poof is running on.
    let current_os = t.get_os().as_str();

    // tiny helper
    let is_linux: bool = current_os == "linux";

    // MUSL
    if is_linux && t.is_musl() && item.contains("musl") {
        // First of all, bonus point if (on linux) the binary is musl and user asked for it.
        score += 2;
    } else if is_linux && !t.is_musl() && item.contains("musl") {
        // less points if the binary (on linux) is musl but user didn't ask for it.
        score -= 2;
    }

    // OPERATING_SYSTEM
    let Some(os_aliases) = OPERATING_SYSTEM.get(current_os) else {
        // If current operating system is not in the OPERATING_SYSTEM hashmap, return -1
        // as deal-breaker. 'None' case happens when the hashmap misses the
        // operating system poof is currently running on. This is unlikely to happen,
        // unless the user is running a built poof on a yet unsupported operating system.
        return -1;
    };
    // Check if this OS matches our current OS.
    // matching_os will hold the matched alias among the values that matched.
    // Aliases are retrieved from OPERATING_SYSTEM using t.get_os() as the key.
    let matching_os: Option<String> = os_aliases
        .iter()
        .find(|alias| contains_alias_token(&item, alias))
        .map(|alias| alias.to_string());
    let found_os: bool = matching_os.is_some();
    if found_os {
        score += 5;
    }

    // CPU_ARCH
    // current_arch is the architecture from the AssetTriple.
    // AssetTriple defaults to the architecture poof is running on.
    let current_arch: &str = t.get_arch().as_str();
    // hotfix to allow 'arm' to be used as a alias for 'armv7'.
    // because Rust ARCH only support 'arm' for all 32 bit arm architectures.
    // we can do this safely because poof targets armv7 gnu/musl hard-float builds.
    let current_arch: &str = if current_arch == "arm" {
        "armv7"
    } else {
        current_arch
    };
    let Some(arch_aliases) = CPU_ARCH.get(current_arch) else {
        // If current architecture is not in the CPU_ARCH hashmap, return -1
        // as deal-breaker. 'None' case happens when the hashmap misses the
        // architecture poof is currently running on. This is unlikely to happen,
        // unless the user is running a built poof on a yet unsupported architecture.
        return -1;
    };
    // Check if architecture matches any alias for our current architecture.
    // matching_arch will hold the alias among the values that matched.
    // Aliases are retrieved from CPU_ARCH using current_arch as the key.
    // If a match is found, matching_arch holds the matched alias;
    // otherwise it stays "unknown".
    let matching_arch: Option<(usize, String)> = arch_aliases
        .iter()
        .enumerate()
        .find(|(_, alias)| contains_alias_token(&item, alias))
        .map(|(idx, alias)| {
            // We also add a base 5 points bonus in case of a match, like for OS matching.
            // Then we add the number of aliases remaining to the score to give more priority
            // to the earlier aliases in the array.
            // This is to avoid false positives for assets that have multiple options for
            // the same architecture, e.g. armv7, armv6 and armhf when running on armv7.
            score = score + 5 + (arch_aliases.len() - idx) as i32;
            (idx, alias.to_string())
        });
    let matching_arch: Option<String> = matching_arch.map(|(_, s)| s);
    let found_arch: bool = matching_arch.is_some();

    // ADDITIONAL CHECKS
    // fix to avoid mismatch between the asset and the target architecture
    // due to 'x86' being a substring of 'x86_64'.
    if (item.contains("x86_64") || item.contains("x86-64")) && current_arch == "x86" {
        return -1;
    }

    // fix to avoid mismatch between the asset and the target architecture
    // due to 'arm' (32 bit) being a substring of 'arm64' and avoid
    // installing 64 bit binaries on 32 bit arm devices.
    if item.contains("arm64") && current_arch == "armv7" {
        return -1;
    }

    // FILENAME_PATTERN
    if !item.contains(".") {
        // if the executable name does not contain a dot, it does not have an extension,
        // so it's likely a compatible non-archived executable. we give it a lower bonus point.
        // keep no-extension support, but require some platform signal
        if !(found_os || found_arch) {
            return -1;
        }
        score += 2;
    } else if found_arch && item.ends_with(matching_arch.as_ref().unwrap()) {
        // if the executable name ends with the matching architecture, we give it a lower bonus point.
        // this is likely a binary that is released as an executable without an archive.
        score += 2;
    } else if found_os && item.ends_with(matching_os.as_ref().unwrap()) {
        // if the executable name ends with the matching operating system, we give it a lower bonus point.
        // this is likely a binary that is released as an executable without an archive.
        score += 2;
    } else if has_extension(&item)
        && SUPPORTED_EXTENSIONS
            .iter()
            .any(|&format| item.ends_with(format))
    {
        // otherwise, check if it's a real extension and among the supported ones.
        // if it does, we give it a higher bonus point, as we explicitly support it, without guessing.
        score += 5;
    } else {
        // if none of the above conditions are met, it's likely not a meaningful binary asset.
        // we discard it by returning -1 as deal-breaker.
        return -1;
    }

    // ADDITIONAL PATCHES
    // Patch assets missing OS label.
    // Usually these are linux x86_64 assets missing the OS tag.
    if !found_os && is_linux {
        score += 1;
    }

    // Patch assets missing architecture label.
    // Usually these are linux x86_64 assets missing the architecture tag.
    if !found_arch && current_arch == "x86_64" {
        score += 1;
    }

    // finally, return the score
    score
}

/// Return an array of the aliases for the current architecture and operating system.
pub fn platforms_strings() -> Vec<String> {
    // get info of the current platform
    let t: AssetTriple = AssetTriple::default();
    let current_arch = t.get_arch().as_str();
    let Some(arch_aliases) = CPU_ARCH.get(current_arch) else {
        return Vec::new();
    };
    let current_os = t.get_os().as_str();
    let Some(os_aliases) = OPERATING_SYSTEM.get(current_os) else {
        return Vec::new();
    };
    // return the platform strings
    let mut s: Vec<String> = Vec::new();
    s.extend(arch_aliases.iter().map(|alias| alias.to_string()));
    s.extend(os_aliases.iter().map(|alias| alias.to_string()));
    s
}

/// Returns `true` if `item` is a checksum file.
fn is_checksum_file(item: &str) -> bool {
    let item = item.to_lowercase();
    item == "checksum.txt"
        || item == "checksums.txt"
        || item.ends_with(".sha256")
        || item.ends_with(".sha256sum")
        || item.ends_with(".sha1")
        || item.ends_with(".sha1sum")
        || item.ends_with(".md5")
        || item.ends_with(".md5sum")
        || item.ends_with(".sha512")
        || item.ends_with(".sha512sum")
        || item.ends_with(".crc32")
        || item.ends_with(".crc64")
        || item.ends_with(".crc")
        || item.ends_with(".sfv")
}

/// Returns `true` if `item` is a signature file.
fn is_signature_file(item: &str) -> bool {
    let item = item.to_lowercase();
    item.ends_with(".asc")
        || item.ends_with(".sig")
        || item.ends_with(".pem")
        || item.ends_with(".minisign")
        || item.ends_with(".pgp")
        || item.ends_with(".gpg")
}

#[cfg(test)]
mod tests;
