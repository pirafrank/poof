//! A tuple of OS, Architecture, and Musl flag.
use std::{
    env::consts::{ARCH, OS},
    fmt::Display,
};

use crate::core::musl::target_prefers_musl;

/// A triple that captures the target OS, CPU architecture, and MUSL preference.
///
/// The default implementation reflects the current build target and the result
/// of [`target_prefers_musl`]. A custom triple can be constructed with [`new`](AssetTriple::new)
/// for testing or cross-compilation scenarios.
pub struct AssetTriple {
    /// Target operating system identifier (e.g. `"linux"`, `"macos"`).
    os: String,
    /// Target CPU architecture identifier (e.g. `"x86_64"`, `"aarch64"`).
    arch: String,
    /// Whether the target prefers musl-linked binaries.
    musl: bool,
}

#[allow(dead_code)]
impl AssetTriple {
    /// Create a new [`AssetTriple`] with explicitly provided `os`, `arch`, and `musl` values.
    pub fn new(os: String, arch: String, musl: bool) -> Self {
        Self { os, arch, musl }
    }

    /// Return the operating system identifier (e.g. `"linux"`, `"macos"`).
    pub fn get_os(&self) -> &String {
        &self.os
    }

    /// Return the CPU architecture identifier (e.g. `"x86_64"`, `"aarch64"`).
    pub fn get_arch(&self) -> &String {
        &self.arch
    }

    /// Return `true` when MUSL-linked assets are preferred over glibc-linked ones.
    pub fn is_musl(&self) -> bool {
        self.musl
    }
}

impl Default for AssetTriple {
    fn default() -> Self {
        Self {
            os: OS.to_string(),
            arch: ARCH.to_string(),
            musl: target_prefers_musl(),
        }
    }
}

impl Display for AssetTriple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "os={}_arch={}_musl={}", self.os, self.arch, self.musl)
    }
}
