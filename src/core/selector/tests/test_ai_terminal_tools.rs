//! Release asset selection for terminal tools on Linux amd64 and macOS arm64.
//!
//! These offline tests simulate both platforms; they do not execute downloaded binaries.

use crate::core::selector::get_triple_compatible_assets;
use crate::models::asset_triple::AssetTriple;

fn assert_selected_assets(fixture: &str, os: &str, arch: &str, expected: &[&str]) {
    let assets: Vec<String> = ron::from_str(fixture).expect("valid release asset fixture");
    let platform = AssetTriple::new(os.to_string(), arch.to_string(), false);
    let selected = get_triple_compatible_assets(&assets, &platform, |asset| asset)
        .expect("release should have matching assets");
    assert_eq!(selected, expected, "unexpected assets for {os}/{arch}");
}

macro_rules! terminal_tool_tests {
    ($tool:ident, $fixture:literal, $linux:literal, $macos:literal) => {
        mod $tool {
            use super::assert_selected_assets;

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

terminal_tool_tests!(
    herdr,
    "assets/herdrdev@herdr.ron",
    "herdr-linux-x86_64",
    "herdr-macos-aarch64"
);
terminal_tool_tests!(
    tuicr,
    "assets/agavra@tuicr.ron",
    "tuicr-0.28.0-x86_64-unknown-linux-gnu.tar.gz",
    "tuicr-0.28.0-aarch64-apple-darwin.tar.gz"
);
terminal_tool_tests!(
    gh_dash,
    "assets/dlvhdr@gh-dash.ron",
    "gh-dash_v4.26.2_linux-amd64",
    "gh-dash_v4.26.2_darwin-arm64"
);
terminal_tool_tests!(
    diffnav,
    "assets/dlvhdr@diffnav.ron",
    "diffnav_Linux_x86_64.tar.gz",
    "diffnav_Darwin_arm64.tar.gz"
);
terminal_tool_tests!(
    delta,
    "assets/dandavison@delta.ron",
    "delta-0.18.2-x86_64-unknown-linux-gnu.tar.gz",
    "delta-0.18.2-aarch64-apple-darwin.tar.gz"
);
terminal_tool_tests!(
    lazygit,
    "assets/jesseduffield@lazygit.ron",
    "lazygit_0.58.1_linux_x86_64.tar.gz",
    "lazygit_0.58.1_darwin_arm64.tar.gz"
);
terminal_tool_tests!(
    yazi,
    "assets/sxyazi@yazi.ron",
    "yazi-x86_64-unknown-linux-gnu.zip",
    "yazi-aarch64-apple-darwin.zip"
);
terminal_tool_tests!(
    mermaid_ascii,
    "assets/AlexanderGrooff@mermaid-ascii.ron",
    "mermaid-ascii_Linux_x86_64.tar.gz",
    "mermaid-ascii_Darwin_arm64.tar.gz"
);
terminal_tool_tests!(
    glow,
    "assets/charmbracelet@glow.ron",
    "glow_2.1.1_Linux_x86_64.tar.gz",
    "glow_2.1.1_Darwin_arm64.tar.gz"
);
