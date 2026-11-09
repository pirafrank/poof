//! Utility functions and macros for terminal tool release asset tests.
//!
//! Provides helper functions and macros for testing terminal tool release asset selection.
//! Contains the `assert_selected_assets` function and the `terminal_tool_tests` macro.
//!
//! This module is used exclusively in the terminal tool release asset tests.
//! It should not be used outside of these tests.

use crate::core::selector::get_triple_compatible_assets;
use crate::models::asset_triple::AssetTriple;

pub fn assert_selected_assets(fixture: &str, os: &str, arch: &str, expected: &[&str]) {
    let assets: Vec<String> = ron::from_str(fixture).expect("valid release asset fixture");
    let platform = AssetTriple::new(os.to_string(), arch.to_string(), false);
    let selected = get_triple_compatible_assets(&assets, &platform, |asset| asset)
        .expect("release should have matching assets");
    assert_eq!(selected, expected, "unexpected assets for {os}/{arch}");
}

/// Generates platform-selection tests for a terminal tool's release assets.
///
/// `$fixture` is the fixture path, and `$linux` and `$macos` are the expected
/// asset names for Linux amd64 and macOS arm64, respectively.
#[macro_export]
macro_rules! terminal_tool_tests {
    ($tool:ident, $fixture:literal, $linux:literal, $macos:literal) => {
        mod $tool {
            use $crate::core::selector::tests::utils::assert_selected_assets;

            #[test]
            fn linux_amd64() {
                assert_selected_assets(include_str!($fixture), "linux", "x86_64", &[$linux]);
            }

            #[test]
            fn macos_arm64() {
                assert_selected_assets(include_str!($fixture), "macos", "aarch64", &[$macos]);
            }
        }
    };
}
