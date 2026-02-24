//! Supported shell types for init and completions commands

use std::fmt;
use std::str::FromStr;

/// A shell supported by poof's `init` and `completions` commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedShell {
    /// Bourne-Again SHell.
    Bash,
    /// Elvish shell.
    Elvish,
    /// Fish shell.
    Fish,
    /// Nushell (also accepted as `nu`).
    Nushell,
    /// PowerShell (also accepted as `pwsh`).
    PowerShell,
    /// Xonsh shell.
    Xonsh,
    /// Z Shell.
    Zsh,
}

/// Error returned when an unrecognised shell name is supplied to [`SupportedShell::from_str`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseShellError {
    input: String,
}

impl ParseShellError {
    fn new(input: impl Into<String>) -> Self {
        Self {
            input: input.into(),
        }
    }
}

impl fmt::Display for ParseShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unsupported shell: '{}'", self.input)
    }
}

impl std::error::Error for ParseShellError {}

impl FromStr for SupportedShell {
    type Err = ParseShellError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bash" => Ok(Self::Bash),
            "elvish" => Ok(Self::Elvish),
            "fish" => Ok(Self::Fish),
            "nushell" | "nu" => Ok(Self::Nushell),
            "powershell" | "pwsh" => Ok(Self::PowerShell),
            "xonsh" => Ok(Self::Xonsh),
            "zsh" => Ok(Self::Zsh),
            _ => Err(ParseShellError::new(s)),
        }
    }
}

impl SupportedShell {
    /// Get all possible shell values
    pub fn possible_values() -> &'static [&'static str] {
        &[
            "bash",
            "elvish",
            "fish",
            "nushell",
            "nu",
            "powershell",
            "pwsh",
            "xonsh",
            "zsh",
        ]
    }
}
