//! Persistently add poof's bin directory to PATH
//! Supports all shells: bash, zsh, fish, elvish, nushell, powershell, xonsh

use std::path::Path;
use std::{fs, io::Write, path::PathBuf};

use log::{error, info};

use crate::files::datadirs::get_bin_dir;
use crate::models::supported_shells::SupportedShell;

/// Get the configuration file path for a given shell
fn get_config_path(shell: SupportedShell, home: &Path) -> PathBuf {
    match shell {
        SupportedShell::Bash => home.join(".bashrc"),
        SupportedShell::Zsh => home.join(".zshrc"),
        SupportedShell::Fish => home.join(".config").join("fish").join("config.fish"),
        SupportedShell::Elvish => home.join(".config").join("elvish").join("rc.elv"),
        SupportedShell::Nushell => home.join(".config").join("nushell").join("env.nu"),
        SupportedShell::PowerShell => {
            // On Linux/macOS: ~/.config/powershell/Microsoft.PowerShell_profile.ps1
            home.join(".config")
                .join("powershell")
                .join("Microsoft.PowerShell_profile.ps1")
        }
        SupportedShell::Xonsh => home.join(".xonshrc"),
    }
}

/// Get the shell name as a string for display purposes
fn shell_name(shell: SupportedShell) -> &'static str {
    match shell {
        SupportedShell::Bash => "bash",
        SupportedShell::Zsh => "zsh",
        SupportedShell::Fish => "fish",
        SupportedShell::Elvish => "elvish",
        SupportedShell::Nushell => "nushell",
        SupportedShell::PowerShell => "powershell",
        SupportedShell::Xonsh => "xonsh",
    }
}

/// Generate the content to add to the shell configuration file
fn generate_config_content(shell: SupportedShell, bin_dir: &str) -> String {
    match shell {
        SupportedShell::Bash | SupportedShell::Zsh | SupportedShell::Elvish => {
            // For eval-based shells, use dynamic approach
            format!(
                "\n# added by poof\neval \"$(poof init --shell {})\"",
                shell_name(shell)
            )
        }
        SupportedShell::PowerShell => {
            // PowerShell uses Invoke-Expression
            "\n# added by poof\nInvoke-Expression (& poof init --shell powershell)".to_string()
        }
        SupportedShell::Fish => {
            // Fish uses direct command
            format!("\n# added by poof\nfish_add_path -p \"{}\"", bin_dir)
        }
        SupportedShell::Nushell => {
            // Nushell uses direct assignment
            format!(
                "\n# added by poof\n$env.PATH = ($env.PATH | prepend \"{}\")",
                bin_dir
            )
        }
        SupportedShell::Xonsh => {
            // Xonsh uses Python-like syntax
            format!("\n# added by poof\n$PATH.insert(0, \"{}\")", bin_dir)
        }
    }
}

/// Check if poof is already enabled in the config file
fn is_already_enabled(config_path: &PathBuf, shell: SupportedShell) -> bool {
    if let Ok(text) = fs::read_to_string(config_path) {
        // Check for the marker comment
        if text.contains("# added by poof") {
            return true;
        }

        // Also check for shell-specific patterns
        match shell {
            SupportedShell::Bash | SupportedShell::Zsh | SupportedShell::Elvish => {
                // Check for eval pattern
                if text.contains(&format!("poof init --shell {}", shell_name(shell))) {
                    return true;
                }
            }
            SupportedShell::PowerShell => {
                if text.contains("poof init --shell powershell") {
                    return true;
                }
            }
            SupportedShell::Fish => {
                if text.contains("fish_add_path") && text.contains("poof") {
                    return true;
                }
            }
            SupportedShell::Nushell => {
                if text.contains("$env.PATH") && text.contains("poof") {
                    return true;
                }
            }
            SupportedShell::Xonsh => {
                if text.contains("$PATH.insert") && text.contains("poof") {
                    return true;
                }
            }
        }
    }
    false
}

/// Get the reload instruction for a given shell
fn get_reload_instruction(shell: SupportedShell, config_path: &Path) -> String {
    match shell {
        SupportedShell::Bash | SupportedShell::Zsh => {
            format!("source {}", config_path.display())
        }
        SupportedShell::Fish => {
            format!("source {}", config_path.display())
        }
        SupportedShell::Elvish => "use rc; rc:reload".to_string(),
        SupportedShell::Nushell => {
            format!("source {}", config_path.display())
        }
        SupportedShell::PowerShell => {
            format!(". {}", config_path.display())
        }
        SupportedShell::Xonsh => {
            format!("source {}", config_path.display())
        }
    }
}

pub fn run(shell: SupportedShell) {
    /* 1 ─ get the directory that holds poof's executables */
    let bin_dir = match get_bin_dir() {
        Some(p) => p,
        None => {
            error!("Cannot locate bin directory");
            return;
        }
    };
    let bin = bin_dir.to_string_lossy();

    /* 2 ─ get HOME directory */
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            error!("Cannot find $HOME");
            return;
        }
    };

    let config_path = get_config_path(shell, &home);

    /* 3 ─ if poof is already enabled, do nothing */
    if is_already_enabled(&config_path, shell) {
        info!("poof already enabled in {}", config_path.display());
        return;
    }

    /* 4 ─ create parent directories if needed */
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Cannot create directory {}: {}", parent.display(), e);
                return;
            }
        }
    }

    /* 5 ─ append the configuration content */
    let content = generate_config_content(shell, &bin);

    let mut file = match fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
    {
        Ok(f) => f,
        Err(e) => {
            error!("Cannot open {}: {}", config_path.display(), e);
            return;
        }
    };

    if writeln!(file, "{}", content).is_err() {
        error!("Could not write to {}", config_path.display());
        return;
    }

    let reload_cmd = get_reload_instruction(shell, &config_path);
    info!(
        "🪄 Added poof to {}. Run `{}` to reload your shell or open a new terminal.",
        config_path.display(),
        reload_cmd
    );
}

// ------------------------------------------------------------------
//                       unit‑tests
// ------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_path_for_all_shells() {
        let home = PathBuf::from("/home/user");

        assert_eq!(
            get_config_path(SupportedShell::Bash, &home),
            home.join(".bashrc")
        );
        assert_eq!(
            get_config_path(SupportedShell::Zsh, &home),
            home.join(".zshrc")
        );
        assert_eq!(
            get_config_path(SupportedShell::Fish, &home),
            home.join(".config/fish/config.fish")
        );
        assert_eq!(
            get_config_path(SupportedShell::Elvish, &home),
            home.join(".config/elvish/rc.elv")
        );
        assert_eq!(
            get_config_path(SupportedShell::Nushell, &home),
            home.join(".config/nushell/env.nu")
        );
        assert_eq!(
            get_config_path(SupportedShell::PowerShell, &home),
            home.join(".config/powershell/Microsoft.PowerShell_profile.ps1")
        );
        assert_eq!(
            get_config_path(SupportedShell::Xonsh, &home),
            home.join(".xonshrc")
        );
    }

    #[test]
    fn test_shell_name_mapping() {
        assert_eq!(shell_name(SupportedShell::Bash), "bash");
        assert_eq!(shell_name(SupportedShell::Zsh), "zsh");
        assert_eq!(shell_name(SupportedShell::Fish), "fish");
        assert_eq!(shell_name(SupportedShell::Elvish), "elvish");
        assert_eq!(shell_name(SupportedShell::Nushell), "nushell");
        assert_eq!(shell_name(SupportedShell::PowerShell), "powershell");
        assert_eq!(shell_name(SupportedShell::Xonsh), "xonsh");
    }

    #[test]
    fn test_generate_config_content_eval_shells() {
        let bin_dir = "/home/user/.local/share/poof/bin";

        // Test bash (eval)
        let bash_content = generate_config_content(SupportedShell::Bash, bin_dir);
        assert!(bash_content.contains("# added by poof"));
        assert!(bash_content.contains("eval \"$(poof init --shell bash)\""));

        // Test zsh (eval)
        let zsh_content = generate_config_content(SupportedShell::Zsh, bin_dir);
        assert!(zsh_content.contains("# added by poof"));
        assert!(zsh_content.contains("eval \"$(poof init --shell zsh)\""));

        // Test elvish (eval)
        let elvish_content = generate_config_content(SupportedShell::Elvish, bin_dir);
        assert!(elvish_content.contains("# added by poof"));
        assert!(elvish_content.contains("eval \"$(poof init --shell elvish)\""));
    }

    #[test]
    fn test_generate_config_content_direct_shells() {
        let bin_dir = "/home/user/.local/share/poof/bin";

        // Test fish (direct)
        let fish_content = generate_config_content(SupportedShell::Fish, bin_dir);
        assert!(fish_content.contains("# added by poof"));
        assert!(fish_content.contains("fish_add_path -p"));
        assert!(fish_content.contains(bin_dir));

        // Test nushell (direct)
        let nushell_content = generate_config_content(SupportedShell::Nushell, bin_dir);
        assert!(nushell_content.contains("# added by poof"));
        assert!(nushell_content.contains("$env.PATH"));
        assert!(nushell_content.contains("prepend"));
        assert!(nushell_content.contains(bin_dir));

        // Test xonsh (direct)
        let xonsh_content = generate_config_content(SupportedShell::Xonsh, bin_dir);
        assert!(xonsh_content.contains("# added by poof"));
        assert!(xonsh_content.contains("$PATH.insert"));
        assert!(xonsh_content.contains(bin_dir));

        // Test powershell (Invoke-Expression)
        let pwsh_content = generate_config_content(SupportedShell::PowerShell, bin_dir);
        assert!(pwsh_content.contains("# added by poof"));
        assert!(pwsh_content.contains("Invoke-Expression"));
        assert!(pwsh_content.contains("poof init --shell powershell"));
    }

    #[test]
    fn test_get_reload_instruction_for_all_shells() {
        let home = PathBuf::from("/home/user");

        let bash_config = get_config_path(SupportedShell::Bash, &home);
        let bash_reload = get_reload_instruction(SupportedShell::Bash, &bash_config);
        assert!(bash_reload.contains("source"));
        assert!(bash_reload.contains(".bashrc"));

        let zsh_config = get_config_path(SupportedShell::Zsh, &home);
        let zsh_reload = get_reload_instruction(SupportedShell::Zsh, &zsh_config);
        assert!(zsh_reload.contains("source"));
        assert!(zsh_reload.contains(".zshrc"));

        let fish_config = get_config_path(SupportedShell::Fish, &home);
        let fish_reload = get_reload_instruction(SupportedShell::Fish, &fish_config);
        assert!(fish_reload.contains("source"));
        assert!(fish_reload.contains("config.fish"));

        let elvish_config = get_config_path(SupportedShell::Elvish, &home);
        let elvish_reload = get_reload_instruction(SupportedShell::Elvish, &elvish_config);
        assert!(elvish_reload.contains("rc:reload"));

        let nushell_config = get_config_path(SupportedShell::Nushell, &home);
        let nushell_reload = get_reload_instruction(SupportedShell::Nushell, &nushell_config);
        assert!(nushell_reload.contains("source"));
        assert!(nushell_reload.contains("env.nu"));

        let pwsh_config = get_config_path(SupportedShell::PowerShell, &home);
        let pwsh_reload = get_reload_instruction(SupportedShell::PowerShell, &pwsh_config);
        assert!(pwsh_reload.starts_with("."));
        assert!(pwsh_reload.contains("Microsoft.PowerShell_profile.ps1"));

        let xonsh_config = get_config_path(SupportedShell::Xonsh, &home);
        let xonsh_reload = get_reload_instruction(SupportedShell::Xonsh, &xonsh_config);
        assert!(xonsh_reload.contains("source"));
        assert!(xonsh_reload.contains(".xonshrc"));
    }
}
