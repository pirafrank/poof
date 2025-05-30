#[derive(Debug, PartialEq)]
pub enum BinaryContainer {
    Zip,
    TarGz,
    TarXz,
    TarBz2,
    Tar,
    Gz,
    Xz,
    Bz2,
    SevenZ,
    Unknown,
}
