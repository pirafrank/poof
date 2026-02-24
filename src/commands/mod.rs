/// Verifies that the poof bin directory is present in PATH.
pub mod check;
/// Empties the poof cache directory.
pub mod clean;
/// Generates shell completion scripts.
pub mod completions;
/// Downloads a GitHub release asset to the current directory.
pub mod download;
/// Persistently adds the poof bin directory to a shell's PATH configuration.
pub mod enable;
/// Displays poof installation and environment information.
pub mod info;
/// Generates a shell-specific init script for PATH setup.
pub mod init;
/// Downloads and installs a GitHub release binary.
pub mod install;
/// Lists installed binaries and their versions.
pub mod list;
/// Sets a specific installed version as the default symlink in PATH.
pub mod make_default;
/// Removes an installed binary and its symlinks.
pub mod uninstall;
/// Removes a binary symlink from the PATH directory.
pub mod unlink;
/// Updates installed binaries to their latest GitHub release.
pub mod update;
/// Shows which binaries are provided by an installed repository.
pub mod what;
/// Shows which repository provides a given binary name.
pub mod which;
