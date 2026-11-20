#[cfg(test)]
mod tests {

    use crate::core::selector::{get_triple_compatible_assets, get_triple_score};
    use crate::models::asset_triple::AssetTriple;

    #[test]
    fn test_never_supported_os() {
        // we are goint to never support tvos
        let platform_triple = AssetTriple::new("tvos".to_string(), "aarch64".to_string(), false);
        // fanta binary, but we should not actually score it because we are on a never-supported OS.
        let score: i32 = get_triple_score("fanta_1.0.0_tvos_arm64.tar.gz", &platform_triple);
        assert_eq!(score, -1);
    }

    #[test]
    fn test_never_supported_arch() {
        // we are goint to never support m68k
        let platform_triple = AssetTriple::new("linux".to_string(), "m68k".to_string(), false);
        // fanta binary, but we should not actually score it because we are on a never-supported arch.
        let score: i32 = get_triple_score("fanta_1.0.0_Linux_m68k.tar.gz", &platform_triple);
        assert_eq!(score, -1);
    }

    #[test]
    fn test_yt_dlp_aarch64_asset_is_incompatible_with_x86_64_linux() {
        let platform = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);

        assert_eq!(get_triple_score("yt-dlp_linux_aarch64.zip", &platform), -1);
    }

    #[test]
    fn test_yt_dlp_selects_only_x86_64_linux_asset() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/yt-dlp@yt-dlp.ron")).unwrap();
        let platform = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);

        assert_eq!(
            get_triple_compatible_assets(&assets, &platform, |asset| asset),
            Some(vec!["yt-dlp_linux".to_string()])
        );
    }

    #[test]
    fn test_rejects_assets_for_other_operating_systems() {
        let linux = AssetTriple::new("linux".into(), "x86_64".into(), false);
        for asset in [
            "tool_macos_x86_64.zip",
            "tool_windows_x86_64.zip",
            "tool_freebsd_x86_64.tar.gz",
            "tool_x86_64_linux_android.tar.gz",
            "yt-dlp_win.zip",
        ] {
            assert_eq!(get_triple_score(asset, &linux), -1, "{asset}");
        }

        let macos = AssetTriple::new("macos".into(), "aarch64".into(), false);
        assert_eq!(get_triple_score("tool_linux_aarch64.zip", &macos), -1);
        assert!(get_triple_score("tool_darwin_aarch64.zip", &macos) > 0);
    }

    #[test]
    fn test_rejects_unsupported_architectures() {
        let platform = AssetTriple::new("linux".into(), "x86_64".into(), false);
        for asset in [
            "tool_linux_sparc64.zip",
            "tool_linux_mips64.zip",
            "tool_linux_wasm32.zip",
            "tool_linux_m68k.zip",
        ] {
            assert_eq!(get_triple_score(asset, &platform), -1, "{asset}");
        }
        assert!(get_triple_score("tool_linux_x86_64.zip", &platform) > 0);
    }

    #[test]
    fn test_yt_dlp_musllinux_selection() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/yt-dlp@yt-dlp.ron")).unwrap();
        let cases = [
            ("x86_64", false, "yt-dlp_linux"),
            ("x86_64", true, "yt-dlp_musllinux"),
            ("aarch64", false, "yt-dlp_linux_aarch64"),
            ("aarch64", true, "yt-dlp_musllinux_aarch64"),
        ];
        for (arch, musl, expected) in cases {
            let platform = AssetTriple::new("linux".into(), arch.into(), musl);
            assert_eq!(
                get_triple_compatible_assets(&assets, &platform, |asset| asset),
                Some(vec![expected.to_string()]),
                "{arch}, musl={musl}"
            );
        }
    }
}
