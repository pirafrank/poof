//! Release asset selection for terminal tools on Linux amd64 and macOS arm64.
//!
//! These offline tests simulate both platforms; they do not execute downloaded binaries.

use crate::terminal_tool_tests;

terminal_tool_tests!(
    mc,
    "assets/minio@mc.ron",
    "mc.linux-amd64.RELEASE.2025-08-13T08-35-41Z",
    "mc.darwin-arm64.RELEASE.2025-08-13T08-35-41Z"
);
