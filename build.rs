use chrono::Utc;
use std::{env, process::Command};

#[macro_use]
extern crate build_cfg;

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

fn c_library_detection() {
    let target = env::var("TARGET").unwrap();
    let is_musl = target.contains("musl");
    let is_gnu = target.contains("gnu");

    // set the c_lib environment variable
    // note: by default, Rust GNU builds target and link against glibc.
    if is_gnu {
        let glibc_version = get_glibc_version().unwrap();
        println!("cargo:rustc-env=C_LIB=glibc v{}", glibc_version);
    } else if is_musl {
        println!("cargo:rustc-env=C_LIB=musl");
    }

    // detect if '-static' is passed for glibc targets
    let rustflags_static = env::var("RUSTFLAGS")
        .map(|f| f.contains("-static"))
        .unwrap_or(false);

    if is_musl || rustflags_static {
        println!("cargo:rustc-cfg=static_linking");
    } else {
        println!("cargo:rustc-cfg=dynamic_linking");
    }

    // avoid warnings about custom cfg macros
    println!("cargo::rustc-check-cfg=cfg(static_linking)");
    println!("cargo::rustc-check-cfg=cfg(dynamic_linking)");
}

#[build_cfg_main]
fn main() {
    // Get the short commit hash
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let git_hash = String::from_utf8(output.stdout).expect("Invalid UTF-8 sequence");

    // today date
    let now = Utc::now();
    let build_date = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Set the environment variables
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_hash.trim());
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // set the linking detection cfg macro
    c_library_detection();
}
