use clap::CommandFactory;
use clap_complete::{generate, Shell};
use clap_complete_nushell::Nushell;
use std::io;

/// Supported shell types for completions
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

    /// Get all possible shell values for clap
    pub fn possible_values() -> Vec<&'static str> {
        vec![
            "bash",
            "elvish",
            "fish",
            "nushell",
            "powershell",
            "xonsh",
            "zsh",
        ]
    }
}

/// Generate shell completions to stdout
pub fn generate_completions(shell: SupportedShell) {
    let mut cmd = crate::Cli::command();
    let bin_name = cmd.get_name().to_string();

    match shell {
        SupportedShell::Bash => generate(Shell::Bash, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::Elvish => generate(Shell::Elvish, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::Fish => generate(Shell::Fish, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::Nushell => generate(Nushell, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::PowerShell => {
            generate(Shell::PowerShell, &mut cmd, &bin_name, &mut io::stdout())
        }
        SupportedShell::Xonsh => generate(Shell::Bash, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::Zsh => generate(Shell::Zsh, &mut cmd, &bin_name, &mut io::stdout()),
    }
}
