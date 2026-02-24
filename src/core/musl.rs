//! MUSL libc detection and preference resolution.
//!
//! On Linux, poof can prefer MUSL-linked assets over glibc-linked ones when
//! the running system does not have glibc (e.g. Alpine Linux). The preference
//! can also be forced by setting `POOF_PREFER_MUSL=1`.

use std::sync::OnceLock;

static CELL: OnceLock<bool> = OnceLock::new();

fn get_default() -> bool {
    let user_value = std::env::var("POOF_PREFER_MUSL")
        .ok()
        .and_then(|v| match v.as_str() {
            "1" | "true" | "TRUE" | "True" => Some(true),
            "0" | "false" | "FALSE" | "False" => Some(false),
            _ => None,
        });
    // if user has set preference, use it; otherwise, detect based on target on Linux
    user_value.unwrap_or_else(target_has_no_glibc) && cfg!(target_os = "linux")
}

/// Return `true` when the current environment should prefer MUSL-linked assets.
///
/// The result is computed once and cached for the lifetime of the process.
/// The value is derived from the `POOF_PREFER_MUSL` environment variable when
/// set; otherwise it is inferred by checking whether the system's `ldd` reports
/// glibc. Always returns `false` on non-Linux targets.
pub fn target_prefers_musl() -> bool {
    *CELL.get_or_init(get_default)
}

#[cfg(target_os = "linux")]
fn get_ldd() -> String {
    use std::process::Command;
    if let Ok(output) = Command::new("ldd").arg("--version").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.to_lowercase()
        } else {
            String::new()
        }
    } else {
        String::new()
    }
}

#[cfg(target_os = "linux")]
fn target_has_no_glibc() -> bool {
    let ldd_output = get_ldd();
    !ldd_output.contains("glibc") && !ldd_output.contains("gnu libc")
}

// On non-Linux targets, we assume glibc is not relevant.
// So we return false, otherwise poof would try to prefer
// musl builds on non-Linux systems, which doesn't make sense.
#[cfg(not(target_os = "linux"))]
fn target_has_no_glibc() -> bool {
    false
}
