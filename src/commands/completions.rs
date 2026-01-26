use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

/// Generate shell completions to stdout
pub fn generate_completions(shell: Shell) {
    let mut cmd = crate::Cli::command();
    generate(shell, &mut cmd, "poof", &mut io::stdout());
}
