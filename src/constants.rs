// package constants
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const DESCRIPTION: &str = "magic manager of pre-built software";
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const THIS_REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

// version constants
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const COMMIT: &str = env!("GIT_COMMIT_HASH");
pub const BUILD_DATE: &str = env!("BUILD_DATE");

// data directory constants
pub const DATA_SUBDIR: &str = "data";
pub const BIN_SUBDIR: &str = "bin";

// other constants
pub const UNKNOWN: &str = "Unknown";
