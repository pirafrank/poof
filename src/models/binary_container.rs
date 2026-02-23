#[derive(Debug, PartialEq)]
pub enum BinaryContainer {
    Zip,
    TarGz,
    TarXz,
    TarBz2,
    TarZstd,
    Tar,
    Gz,
    Xz,
    Bz2,
    Zstd,
    SevenZ,
    Unknown,
}
