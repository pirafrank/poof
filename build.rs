//! Build script for poof.
//!
//! Sets compile-time environment variables: git commit hash, build date,
//! C library name/version, and linking mode (static vs dynamic).

use chrono::Utc;
use std::{env, process::Command};

#[macro_use]
extern crate build_cfg;

/// Returns the glibc version string if targeting GNU, otherwise returns `None`.
fn get_glibc_version() -> Option<String> {
    // Set the glibc version if applicable
    if build_cfg!(target_env = "gnu") {
        let ver = glibc_version::get_version().unwrap();
        let glibc_version = format!("{}.{}", ver.major, ver.minor);
        println!("cargo:rustc-env=GLIBC_VERSION={}", glibc_version);
        Some(glibc_version)
    } else {
        println!("cargo:rustc-env=GLIBC_VERSION=");
        None
    }
}

/// Detects the C library for the target triple and sets linker configuration flags.
fn c_library_detection() {
    let target = env::var("TARGET").unwrap();
    let is_musl = target.contains("musl");
    let is_gnu = target.contains("gnu");
    let is_darwin = target.contains("darwin");
    let is_freebsd = target.contains("freebsd");

    // set the c_lib environment variable
    // note: by default, Rust GNU builds target and link against glibc.
    if is_gnu {
        let glibc_version = get_glibc_version().unwrap();
        println!("cargo:rustc-env=C_LIB=glibc v{}", glibc_version);
    } else if is_musl {
        println!("cargo:rustc-env=C_LIB=musl");
    } else if is_darwin {
        println!("cargo:rustc-env=C_LIB=libSystem");
    } else if is_freebsd {
        println!("cargo:rustc-env=C_LIB=libc");
    }

    // detect if '-static' is passed for glibc targets
    let rustflags_static = env::var("RUSTFLAGS")
        .map(|f| f.contains("-static"))
        .unwrap_or(false);

    if is_musl || rustflags_static {
        println!("cargo:rustc-cfg=static_linking");
    } else {
        // gnu is dynamically linked by default.
        // libSystem on macOS can only be linked statically as Apple
        // does not provide a static version of system libraries.
        // On x86_64-unknown-freebsd, dynamic linking is the default,
        // but you can use '-static' to link statically.
        println!("cargo:rustc-cfg=dynamic_linking");
    }

    // avoid warnings about custom cfg macros
    println!("cargo::rustc-check-cfg=cfg(static_linking)");
    println!("cargo::rustc-check-cfg=cfg(dynamic_linking)");
}

/// Build entry point: embeds git hash, build date, and linking metadata into the binary.
#[build_cfg_main]
fn main() {
    // GIT_COMMIT_HASH: use the env var when set (e.g. injected by the Nix flake) so
    // the build does not need git in the sandbox.  Falls back to `git rev-parse HEAD`
    // for normal cargo builds where git is available.
    let git_hash = env::var("GIT_COMMIT_HASH").unwrap_or_else(|_| {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("Cannot execute git command");
        String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 sequence")
            .trim()
            .to_string()
    });

    // BUILD_DATE: use the env var when set (e.g. injected by the Nix flake for
    // reproducibility).  Falls back to the current UTC time for normal cargo builds.
    let build_date = env::var("BUILD_DATE").unwrap_or_else(|_| {
        let now = Utc::now();
        now.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    });

    // Set the environment variables
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_hash);
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // set the linking detection cfg macro
    c_library_detection();
}
