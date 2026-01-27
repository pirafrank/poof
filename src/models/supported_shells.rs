//! Supported shell types for init and completions commands

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedShell {
    Bash,
    Elvish,
    Fish,
    Nushell,
    PowerShell,
    Xonsh,
    Zsh,
}

impl SupportedShell {
    /// Parse a shell name string into a SupportedShell variant
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Self::Bash),
            "elvish" => Some(Self::Elvish),
            "fish" => Some(Self::Fish),
            "nushell" | "nu" => Some(Self::Nushell),
            "powershell" | "pwsh" => Some(Self::PowerShell),
            "xonsh" => Some(Self::Xonsh),
            "zsh" => Some(Self::Zsh),
            _ => None,
        }
    }

    /// Get all possible shell values
    pub fn possible_values() -> Vec<&'static str> {
        vec![
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
