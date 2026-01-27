use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

/// Generate shell completions to stdout
pub fn generate_completions(shell: Shell) {
    let mut cmd = crate::Cli::command();
    let bin_name = cmd.get_name().to_string();
    generate(shell, &mut cmd, &bin_name, &mut io::stdout());
}
