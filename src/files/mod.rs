/// Archive extraction: tar, gz, xz, bz2, zstd, zip.
pub mod archives;
/// Platform-specific data, bin, cache, and config directory resolution.
pub mod datadirs;
/// Filesystem helpers: find executables, copy files, create symlinks.
pub mod filesys;
/// Binary format detection via magic-number (file-signature) inspection.
pub mod magic;
/// Filename and extension utilities shared across the crate.
pub mod utils;
