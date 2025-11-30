//! Poof - A zero-config, zero-install manager for pre-built software
//!
//! This library provides the core functionality for the poof binary manager.

pub mod github;
pub mod models;
pub mod utils;

// Re-export commonly used items
pub use github::client;
pub use github::models::{Release, ReleaseAsset};
