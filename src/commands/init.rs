use std::io::Write;

use crate::models::supported_shells::SupportedShell;

/// Generate shell-specific init script to add poof bin directory to PATH
pub fn generate_init_script(shell: SupportedShell) {
    let bin_dir = match crate::files::datadirs::get_bin_dir() {
        Some(dir) => dir.to_string_lossy().to_string(),
        None => {
            eprintln!("Error: Could not determine poof bin directory");
            std::process::exit(1);
        }
    };

    let mut stdout = std::io::stdout().lock();

    match shell {
        SupportedShell::Bash | SupportedShell::Zsh => {
            // POSIX-compatible shells
            writeln!(stdout, "export PATH=\"{}:$PATH\"", bin_dir).unwrap();
        }
        SupportedShell::Fish => {
            // Fish shell uses different syntax
            // docs: https://fishshell.com/docs/current/cmds/fish_add_path.html
            // "Directories are added in the order they are given, and they
            // are prepended to the path unless --append is given."
            writeln!(stdout, "fish_add_path -p \"{}\"", bin_dir).unwrap();
        }
        SupportedShell::Elvish => {
            // Elvish shell syntax
            // docs: https://elv.sh/learn/tour.html#changing-path
            writeln!(stdout, "set paths = [\"{}\" $@paths]", bin_dir).unwrap();
        }
        SupportedShell::Nushell => {
            // Nushell syntax
            // docs: https://www.nushell.sh/book/environment.html#env-var-assignment
            writeln!(stdout, "$env.PATH = ($env.PATH | prepend \"{}\")", bin_dir).unwrap();
        }
        SupportedShell::PowerShell => {
            // PowerShell syntax
            // docs: https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_environment_variables?view=powershell-7.5
            writeln!(
                stdout,
                "$env:PATH = \"{}$([System.IO.Path]::PathSeparator)$env:PATH\"",
                bin_dir
            )
            .unwrap();
        }
        SupportedShell::Xonsh => {
            // Xonsh shell (Python-based, uses Python syntax)
            // docs: https://pypi.org/project/xontrib-cheatsheet/
            writeln!(stdout, "$PATH.insert(0, \"{}\")", bin_dir).unwrap();
        }
    }
}
