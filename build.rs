use chrono::Utc;
use std::process::Command;

#[cfg(target_env = "gnu")]
fn glibc_ver() -> String {
    let ver = glibc_version::get_version().unwrap();
    format!("\nDynamically linked against glibc v{}.{}", ver.major, ver.minor)
}

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

    // Set the glibc version if applicable
    #[cfg(target_env = "gnu")]
    {
        let glibc_version = glibc_ver();
        println!("cargo:rustc-env=GLIBC_VERSION={}", glibc_version);
    }
    #[cfg(not(target_env = "gnu"))]
    {
        println!("cargo:rustc-env=GLIBC_VERSION={}", "");
    }
}
