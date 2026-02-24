//! Archive / container format enumeration.

/// Identifies the container or compression format of a downloaded release asset.
#[derive(Debug, PartialEq)]
pub enum BinaryContainer {
    /// ZIP archive (`.zip`).
    Zip,
    /// Gzip-compressed tar archive (`.tar.gz` / `.tgz`).
    TarGz,
    /// XZ-compressed tar archive (`.tar.xz` / `.txz`).
    TarXz,
    /// Bzip2-compressed tar archive (`.tar.bz2` / `.tbz` / `.tbz2`).
    TarBz2,
    /// Zstandard-compressed tar archive (`.tar.zst` / `.tzst`).
    TarZstd,
    /// Uncompressed tar archive (`.tar`).
    Tar,
    /// Gzip-compressed single file (`.gz`).
    Gz,
    /// XZ-compressed single file (`.xz`).
    Xz,
    /// Bzip2-compressed single file (`.bz2`).
    Bz2,
    /// Zstandard-compressed single file (`.zst`).
    Zstd,
    /// 7-Zip archive (`.7z`).
    SevenZ,
    /// Format could not be determined.
    Unknown,
}
