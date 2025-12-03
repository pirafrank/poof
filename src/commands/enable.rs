//! sibellavia: persistently add poof's bin directory to PATH
//! we can evaluate using `anyhow` instead!
//! Also, for now we are not considering Windows.
//! TODO: add support for Windows.

use std::{fs::OpenOptions, io::Write, path::PathBuf};

use log::{error, info};

use crate::files::datadirs::get_bin_dir;

pub fn run() {
    /* 1 ─ get the directory that holds poof's executables */
    let bin_dir = match get_bin_dir() {
        Some(p) => p,
        None => {
            error!("Cannot locate bin directory");
            return;
        }
    };
    let bin = bin_dir.to_string_lossy();

    /* 2 ─ pick which startup script (.bashrc or .zshrc) to modify */
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            error!("Cannot find $HOME");
            return;
        }
    };

    let shell = std::env::var("SHELL").unwrap_or_default();
    let rc: PathBuf = if shell.ends_with("zsh") {
        home.join(".zshrc")
    } else {
        home.join(".bashrc")
    };

    /* 3 ─ if the PATH line is already there, do nothing */
    if let Ok(text) = std::fs::read_to_string(&rc) {
        if text.contains(bin.as_ref()) {
            info!("poof already enabled in {}", rc.display());
            return;
        }
    }

    /* 4 ─ append the export line */
    let mut file = match OpenOptions::new().create(true).append(true).open(&rc) {
        Ok(f) => f,
        Err(e) => {
            error!("Cannot open {}: {}", rc.display(), e);
            return;
        }
    };

    if writeln!(file, "\n# added by poof\nexport PATH=\"{}:$PATH\"", bin).is_err() {
        error!("Could not write to {}", rc.display());
        return;
    }

    info!(
        "🪄 Added poof to {}.\n   Run `source {0}` or open a new terminal.",
        rc.display()
    );
}

// ------------------------------------------------------------------
//                       unit‑tests
// I'm not sure a test suite is provided, I couldn't find one
// so for now I'm just dropping some unit tests here
// ------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::run;
    use crate::files::datadirs::get_bin_dir;
    use std::fs;
    use tempfile::TempDir;

    /// prepare HOME and XDG_DATA_HOME, then create the real bin dir
    /// Returns the bin directory path
    fn create_fake_bin(home: &TempDir) -> std::path::PathBuf {
        // point datadirs::get_bin_dir() at our temp dir
        temp_env::with_vars(
            [
                ("HOME", Some(home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(home.path().to_str().unwrap())),
            ],
            || {
                // now this is "<temp>/poof/bin"
                let bin = get_bin_dir().unwrap();
                fs::create_dir_all(&bin).unwrap();
                bin
            },
        )
    }

    /// read the contents of the given rc‐file
    fn get_rc_contents(rc: &std::path::Path) -> String {
        fs::read_to_string(rc).unwrap_or_default()
    }

    #[test]
    /// test that bashrc is written to and running twice doesn't duplicate
    fn bashrc_idempotent() {
        let temp_home = TempDir::new().unwrap();
        let _bin = create_fake_bin(&temp_home);

        temp_env::with_vars(
            [
                ("HOME", Some(temp_home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(temp_home.path().to_str().unwrap())),
                ("SHELL", Some("/bin/bash")),
            ],
            || {
                // run twice for idempotence
                run();
                run();
            },
        );

        let rc_path = temp_home.path().join(".bashrc");
        let contents = get_rc_contents(&rc_path);

        // Build the exact expected export from the same helper
        temp_env::with_vars(
            [
                ("HOME", Some(temp_home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(temp_home.path().to_str().unwrap())),
            ],
            || {
                let binding = get_bin_dir().unwrap();
                let bin = binding.to_string_lossy();
                let expected = format!("export PATH=\"{}:$PATH\"", bin);

                assert_eq!(
                    contents.matches(&expected).count(),
                    1,
                    "export line should appear exactly once"
                );
            },
        );
    }

    #[test]
    /// test that zshrc is written to
    fn writes_to_zshrc() {
        let temp_home = TempDir::new().unwrap();
        let _bin = create_fake_bin(&temp_home);

        temp_env::with_vars(
            [
                ("HOME", Some(temp_home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(temp_home.path().to_str().unwrap())),
                ("SHELL", Some("/usr/bin/zsh")),
            ],
            || {
                run();
            },
        );

        let rc_path = temp_home.path().join(".zshrc");
        let contents = get_rc_contents(&rc_path);
        assert!(
            contents.contains("export PATH="),
            ".zshrc should contain an export line"
        );
    }

    #[test]
    /// test that zshrc is written to and running twice doesn't duplicate
    fn zsh_idempotent() {
        let temp_home = TempDir::new().unwrap();
        let _bin = create_fake_bin(&temp_home);

        temp_env::with_vars(
            [
                ("HOME", Some(temp_home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(temp_home.path().to_str().unwrap())),
                ("SHELL", Some("/usr/bin/zsh")),
            ],
            || {
                run();
                run();
            },
        );

        let rc_path = temp_home.path().join(".zshrc");
        let contents = get_rc_contents(&rc_path);

        temp_env::with_vars(
            [
                ("HOME", Some(temp_home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(temp_home.path().to_str().unwrap())),
            ],
            || {
                let binding = get_bin_dir().unwrap();
                let bin = binding.to_string_lossy();
                let line = format!("export PATH=\"{}:$PATH\"", bin);

                assert_eq!(
                    contents.matches(&line).count(),
                    1,
                    "zsh idempotence: export line should appear exactly once"
                );
            },
        );
    }

    #[test]
    /// test that unknown shell defaults to bash
    fn unknown_shell_defaults_to_bash() {
        let temp_home = TempDir::new().unwrap();
        let _bin = create_fake_bin(&temp_home);

        temp_env::with_vars(
            [
                ("HOME", Some(temp_home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(temp_home.path().to_str().unwrap())),
                ("SHELL", None), // Remove SHELL var
            ],
            || {
                run();
            },
        );

        let contents = get_rc_contents(&temp_home.path().join(".bashrc"));
        assert!(
            contents.contains("export PATH="),
            "unknown-shell fallback should write to .bashrc"
        );
    }

    #[test]
    /// test that existing rc file content is preserved
    fn preserves_existing_content() {
        let temp_home = TempDir::new().unwrap();
        let _bin = create_fake_bin(&temp_home);

        // Pre-seed .bashrc
        let rc_path = temp_home.path().join(".bashrc");
        fs::write(&rc_path, "PRE_EXISTING_LINE\n").unwrap();

        temp_env::with_vars(
            [
                ("HOME", Some(temp_home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(temp_home.path().to_str().unwrap())),
                ("SHELL", Some("/bin/bash")),
            ],
            || {
                run();
            },
        );

        let contents = get_rc_contents(&rc_path);

        temp_env::with_vars(
            [
                ("HOME", Some(temp_home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(temp_home.path().to_str().unwrap())),
            ],
            || {
                let binding = get_bin_dir().unwrap();
                let bin = binding.to_string_lossy();

                assert!(
                    contents.contains("PRE_EXISTING_LINE"),
                    "existing content must be preserved"
                );
                assert!(
                    contents.contains(&format!("export PATH=\"{}:$PATH\"", bin)),
                    "export line must be appended"
                );
            },
        );
    }

    #[test]
    /// test that comment marker is added to rc file
    fn adds_comment_marker() {
        let temp_home = TempDir::new().unwrap();
        let _bin = create_fake_bin(&temp_home);

        temp_env::with_vars(
            [
                ("HOME", Some(temp_home.path().to_str().unwrap())),
                ("XDG_DATA_HOME", Some(temp_home.path().to_str().unwrap())),
                ("SHELL", Some("/bin/bash")),
            ],
            || {
                run();
            },
        );

        let contents = get_rc_contents(&temp_home.path().join(".bashrc"));
        assert!(
            contents.contains("# added by poof"),
            "comment marker must be present"
        );
    }
}
