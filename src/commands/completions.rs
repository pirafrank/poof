use clap::CommandFactory;
use clap_complete::{generate, Shell};
use clap_complete_nushell::Nushell;
use std::io;

use crate::cli::Cli;
use crate::models::supported_shells::SupportedShell;

/// Generate shell completions to stdout
pub fn generate_completions(shell: SupportedShell) {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    match shell {
        SupportedShell::Bash => generate(Shell::Bash, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::Elvish => generate(Shell::Elvish, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::Fish => generate(Shell::Fish, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::Nushell => generate(Nushell, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::PowerShell => {
            generate(Shell::PowerShell, &mut cmd, &bin_name, &mut io::stdout())
        }
        // Xonsh can source bash completions via its bash foreign function interface.
        // Native xonsh completion generation is not available in clap_complete.
        SupportedShell::Xonsh => generate(Shell::Bash, &mut cmd, &bin_name, &mut io::stdout()),
        SupportedShell::Zsh => generate(Shell::Zsh, &mut cmd, &bin_name, &mut io::stdout()),
    }
}
