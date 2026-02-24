//! Compile-time and runtime constants used throughout the application.

/// The application name, sourced from `Cargo.toml`.
pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
/// A short human-readable description of the application.
pub const DESCRIPTION: &str = "magic manager of pre-built software";
/// The author(s) of the application, sourced from `Cargo.toml`.
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
/// The canonical URL of the upstream repository.
pub const THIS_REPO_URL: &str = env!("CARGO_PKG_REPOSITORY");

/// The current application version, sourced from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The git commit hash captured at build time by `build.rs`.
pub const COMMIT: &str = env!("GIT_COMMIT_HASH");
/// The date the binary was compiled, captured at build time by `build.rs`.
pub const BUILD_DATE: &str = env!("BUILD_DATE");
/// The C standard library used when compiling, captured at build time by `build.rs`.
pub const COMPILE_C_LIB: &str = env!("C_LIB");

/// Sub-directory name under the application's data root used to store installed binaries.
pub const DATA_SUBDIR: &str = "data";
/// Sub-directory name that is added to `PATH` and holds symlinks to installed binaries.
pub const BIN_SUBDIR: &str = "bin";
/// Sub-directory name used to namespace GitHub-hosted repositories inside the data root.
pub const GITHUB_SUBDIR: &str = "github.com";

/// All archive and compression extensions recognised by the asset selector.
///
/// Multi-part extensions (e.g. `.tar.gz`) **must** appear before their single-part
/// counterparts (e.g. `.gz`) so that the longest match wins during extension stripping.
pub const SUPPORTED_EXTENSIONS: [&str; 15] = [
    ".tar.gz", ".tgz", ".tar.xz", ".txz", ".tar.bz2", ".tbz", ".tbz2", ".zip", ".tar", ".gz",
    ".xz", ".bz2", ".tar.zst", ".tzst", ".zst",
];

/// Sentinel string returned when a value cannot be determined at runtime.
pub const UNKNOWN: &str = "Unknown";

/// Characters used as separators inside release asset file names (e.g. `mytool_1.0.0-linux`).
pub const FILENAME_SEPARATORS: [&str; 3] = ["_", "-", "."];
