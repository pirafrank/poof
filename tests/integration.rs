//! Integration tests for stateful commands
//! This file serves as the main entry point for all integration tests

#[path = "common/mod.rs"]
mod common;

#[path = "integration/commands/check.rs"]
mod check;
#[path = "integration/commands/help.rs"]
mod help;
#[path = "integration/commands/info.rs"]
mod info;
#[path = "integration/commands/version.rs"]
mod version;

#[path = "integration/download.rs"]
mod download;
#[path = "integration/enable.rs"]
mod enable;
#[path = "integration/error_handling.rs"]
mod error_handling;
#[path = "integration/install.rs"]
mod install;
#[path = "integration/list.rs"]
mod list;
#[path = "integration/update.rs"]
mod update;
#[path = "integration/use.rs"]
mod r#use;
