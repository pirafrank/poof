//! A tuple of OS, Architecture, and Musl flag.
use std::{
    env::consts::{ARCH, OS},
    fmt::Display,
};

use crate::core::musl::target_prefers_musl;

pub struct AssetTriple {
    os: String,
    arch: String,
    musl: bool,
}

#[allow(dead_code)]
// Allowing dead code to have 'new' used for tests.
impl AssetTriple {
    // Create a new AssetTriple instance with the given OS, ARCH, and MUSL.
    pub fn new(os: String, arch: String, musl: bool) -> Self {
        Self { os, arch, musl }
    }

    pub fn get_os(&self) -> &String {
        &self.os
    }

    pub fn get_arch(&self) -> &String {
        &self.arch
    }

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
