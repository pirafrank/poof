//! Integration tests for stateful commands
//! This file serves as the main entry point for all integration tests

#[path = "common/mod.rs"]
mod common;

#[path = "integration/command_handling/claps.rs"]
mod claps;
#[path = "integration/command_handling/verbose_flags.rs"]
mod verbose_flags;
#[path = "integration/commands/check.rs"]
mod check;
#[path = "integration/commands/help.rs"]
mod help;
#[path = "integration/commands/info.rs"]
mod info;
#[path = "integration/commands/version.rs"]
mod version;

#[path = "integration/commands/download.rs"]
mod download;
#[path = "integration/commands/enable.rs"]
mod enable;
#[path = "integration/commands/install.rs"]
mod install;
#[path = "integration/commands/list.rs"]
mod list;
#[path = "integration/commands/update.rs"]
mod update;
#[path = "integration/commands/use.rs"]
mod r#use;
