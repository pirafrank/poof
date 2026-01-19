#![allow(unused)]
use clap::{ArgAction, Command, CommandFactory};
use clap_mangen::Man;
use std::fs;
use std::io::{Result, Write};
use std::path::PathBuf;

// Reconstruct crate module structure for the example to support src/cli.rs inclusion
#[path = "../src/constants.rs"]
mod constants;

// Import platform_info.rs as a top-level module
#[allow(dead_code)]
#[path = "../src/core/platform_info.rs"]
pub mod platform_info_impl;

// Define core module acting as a namespace
#[allow(dead_code)]
mod core {
    pub use super::platform_info_impl as platform_info;
}

// Import string.rs as a top-level module
#[allow(dead_code)]
#[path = "../src/utils/string.rs"]
pub mod string_impl;

// Define utils module acting as a namespace
#[allow(dead_code)]
mod utils {
    pub use super::string_impl as string;
}

// Include cli module
#[path = "../src/cli.rs"]
mod cli;

use cli::Cli;

fn render_subcommand(sub: &Command, buf: &mut Vec<u8>) -> Result<()> {
    // Subsection title: Command Name
    writeln!(buf, ".SS \"{}\"", sub.get_name())?;

    // Description
    if let Some(about) = sub.get_about() {
        writeln!(buf, "{}", about)?;
    }
    writeln!(buf)?;

    // Arguments / Options
    let args: Vec<_> = sub.get_arguments().collect();

    for arg in args {
        writeln!(buf, ".TP")?;

        let mut opt_str = String::new();

        // Short flag
        if let Some(s) = arg.get_short() {
            opt_str.push_str(&format!("\\fB-{}\\fR", s));
        }

        // Long flag
        if let Some(l) = arg.get_long() {
            if !opt_str.is_empty() {
                opt_str.push_str(", ");
            }
            opt_str.push_str(&format!("\\fB--{}\\fR", l));
        }

        // Determine if argument takes a value
        let action = arg.get_action();
        let takes_value = !matches!(
            action,
            ArgAction::SetTrue
                | ArgAction::SetFalse
                | ArgAction::Help
                | ArgAction::Version
                | ArgAction::Count
        );

        // Positional or value indication
        if opt_str.is_empty() {
            // Positional argument
            opt_str.push_str(&format!("\\fI<{}>\\fR", arg.get_id()));
        } else if takes_value {
            if let Some(val_names) = arg.get_value_names() {
                // Options with values
                for val in val_names {
                    opt_str.push_str(&format!(" \\fI<{}>\\fR", val));
                }
            }
        }

        writeln!(buf, "{}", opt_str)?;

        // Help text
        if let Some(help) = arg.get_help() {
            writeln!(buf, "{}", help)?;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // Determine the output directory
    let mut out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    out_dir.push("man");

    // Create the directory if it doesn't exist
    fs::create_dir_all(&out_dir)?;

    // Build the clap app
    let mut cmd = Cli::command();
    cmd.build(); // Ensure arguments are propagated/finalized

    // The man page name
    let app_name = cmd.get_name().to_string(); // Clone name before cmd is borrowed/moved
    let file_name = format!("{}.1", app_name);
    let mut file_path = out_dir.clone();
    file_path.push(&file_name);

    // Create a clone for the main page generation where we hide subcommands
    // to suppress the default table.
    let mut main_cmd = cmd.clone();
    for sub in main_cmd.get_subcommands_mut() {
        *sub = std::mem::take(sub).hide(true);
    }

    // Generate the main man page structure
    let mut buffer: Vec<u8> = Default::default();
    Man::new(main_cmd).render(&mut buffer)?;

    // Append our custom detailed subcommands section
    writeln!(&mut buffer, ".SH \"SUBCOMMANDS\"")?;

    for sub in cmd.get_subcommands() {
        render_subcommand(sub, &mut buffer)?;
    }

    // Write to file
    fs::write(&file_path, buffer)?;

    println!("Man page generated at: {}", file_path.display());
    Ok(())
}
