//! A tuple of OS, Architecture, Endianness, and Musl.
use std::env::consts::{ARCH, OS};

#[cfg(not(target_env = "musl"))]
const MUSL: bool = false;
#[cfg(target_env = "musl")]
const MUSL: bool = true;

pub struct AssetTriple {
    os: String,
    arch: String,
    musl: bool,
}

#[allow(dead_code)]
impl AssetTriple {
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
            musl: MUSL, // default to target build of poof, user can override
        }
    }
}

impl std::fmt::Display for AssetTriple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "os={}_arch={}_musl={}", self.os, self.arch, self.musl)
    }
}
