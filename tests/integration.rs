//! Integration tests for stateful commands
//! This file serves as the main entry point for all integration tests

#[path = "common/mod.rs"]
mod common;

#[path = "integration/list.rs"]
mod list;
#[path = "integration/make_default.rs"]
mod make_default;
#[path = "integration/enable.rs"]
mod enable;
#[path = "integration/error_handling.rs"]
mod error_handling;
#[path = "integration/download.rs"]
mod download;
#[path = "integration/install.rs"]
mod install;
#[path = "integration/update.rs"]
mod update;
