use std::process::Command;
use chrono::Utc;

fn main() {
    // Get the short commit hash
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let git_hash = String::from_utf8(output.stdout).expect("Invalid UTF-8 sequence");

    // today date
    let now = Utc::now();
    let build_date = now.format("%Y-%m-%d %H:%M:%S UTC").to_string();

    // Set the environment variables
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_hash.trim());
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);
}
