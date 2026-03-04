#[cfg(test)]
mod tests {

    use crate::core::selector::get_triple_compatible_assets;
    use crate::models::asset_triple::AssetTriple;

    #[test]
    fn test_linux_x86_64_glibc() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/jesseduffield@lazygit.ron")).unwrap();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&assets, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("lazygit_0.58.1_linux_x86_64.tar.gz"));
    }

    #[test]
    fn test_linux_x86_64_musl() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/lsd-rs@lsd.ron")).unwrap();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86_64".to_string(), true);
        let binaries = get_triple_compatible_assets(&assets, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("lsd-v1.2.0-x86_64-unknown-linux-musl.tar.gz"));
    }

    #[test]
    fn test_linux_aarch64_glibc() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/bootandy@dust.ron")).unwrap();
        let platform_triple = AssetTriple::new("linux".to_string(), "aarch64".to_string(), false);
        let binaries = get_triple_compatible_assets(&assets, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("dust-v1.2.4-aarch64-unknown-linux-gnu.tar.gz"));
    }

    #[test]
    fn test_linux_aarch64_musl() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/bootandy@dust.ron")).unwrap();
        let platform_triple = AssetTriple::new("linux".to_string(), "aarch64".to_string(), true);
        let binaries = get_triple_compatible_assets(&assets, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("dust-v1.2.4-aarch64-unknown-linux-musl.tar.gz"));
    }

    #[test]
    fn test_linux_i686_glibc() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/sharkdp@fd.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("fd-v10.3.0-i686-unknown-linux-gnu.tar.gz"));
    }

    #[test]
    fn test_linux_i686_musl() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/sharkdp@fd.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86".to_string(), true);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("fd-v10.3.0-i686-unknown-linux-musl.tar.gz"));
    }

    #[test]
    fn test_linux_armv7_glibc() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/ClementTsang@bottom.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "arm".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("bottom_armv7-unknown-linux-gnueabihf.tar.gz"));
    }

    #[test]
    fn test_linux_armv7_musl() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/ClementTsang@bottom.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "arm".to_string(), true);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("bottom_armv7-unknown-linux-musleabihf.tar.gz"));
    }

    #[test]
    fn test_linux_riscv64_glibc() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/sxyazi@yazi.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "riscv64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("yazi-riscv64gc-unknown-linux-gnu.zip"));
    }

    #[test]
    fn test_linux_s390x_glibc() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/gokcehan@lf.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "s390x".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("lf-linux-s390x.tar.gz"));
    }

    #[test]
    fn test_linux_powerpc64le_glibc() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/gokcehan@lf.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "powerpc64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("lf-linux-ppc64le.tar.gz"));
    }

    #[test]
    fn test_linux_loongarch64_glibc() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/ClementTsang@bottom.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple =
            AssetTriple::new("linux".to_string(), "loongarch64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("bottom_loongarch64-unknown-linux-gnu.tar.gz"));
    }

    #[test]
    fn test_macos_aarch64() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/helix-editor@helix.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("macos".to_string(), "aarch64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("helix-25.07.1-aarch64-macos.tar.xz"));
    }

    #[test]
    fn test_macos_x86_64() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/helix-editor@helix.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("macos".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("helix-25.07.1-x86_64-macos.tar.xz"));
    }

    //
    // more tests for uncommon and edge cases follows.
    //

    #[test]
    fn test_linux_x86_64_compatible_binary_without_extension() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/shshemi@tabiew.ron")).unwrap();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&assets, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("tw-x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn test_linux_arm_glibc_gnueabihf() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/bootandy@dust.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "arm".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("dust-v1.2.4-arm-unknown-linux-gnueabihf.tar.gz"));
    }

    #[test]
    fn test_linux_arm_glibc_musleabi() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/bootandy@dust.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "arm".to_string(), true);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("dust-v1.2.4-arm-unknown-linux-musleabi.tar.gz"));
    }

    #[test]
    fn test_linux_i686_glibc_uncommon_name() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/jesseduffield@lazygit.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("lazygit_0.58.1_linux_32-bit.tar.gz"));
    }

    #[test]
    fn test_linux_x86_glibc_no_i_prefix() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/gokcehan@lf.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("lf-linux-386.tar.gz"));
    }

    #[test]
    fn test_linux_s390x_unarchived_binary() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/direnv@direnv.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "s390x".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("direnv.linux-s390x"));
    }

    #[test]
    fn test_linux_armv7_with_armv6_glibc_asset() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/jesseduffield@lazygit.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "arm".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("lazygit_0.58.1_linux_armv6.tar.gz"));
    }

    #[test]
    fn test_linux_arm_no_suffix_glibc_asset() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/charmbracelet@freeze.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "arm".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("freeze_0.2.2_Linux_arm.tar.gz"));
    }

    #[test]
    fn test_linux_i586_no_suffix_glibc_asset() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/charmbracelet@freeze.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("freeze_0.2.2_Linux_i386.tar.gz"));
    }

    #[test]
    fn test_linux_arm64_no_suffix_glibc_asset() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/charmbracelet@freeze.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "aarch64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains("freeze_0.2.2_Linux_arm64.tar.gz"));
    }

    #[test]
    fn test_linux_x86_64_missing_os_label() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/sharkdp@fd.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert_eq!(binaries.len(), 1);
        assert!(binaries[0].contains("fd-v10.3.0-x86_64-unknown-linux-gnu.tar.gz"));
    }

    #[test]
    fn test_linux_x86_64_missing_arch_label() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/jwt-rs@jwt-ui.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert_eq!(binaries.len(), 1);
        assert!(binaries[0].contains("jwtui-linux.tar.gz"));
    }

    #[test]
    fn test_linux_armv7_missing_os_label() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/jwt-rs@jwt-ui.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "arm".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("jwtui-armv7-gnu.tar.gz"));
    }

    #[test]
    fn test_linux_armv7_musl_missing_os_label() {
        let assets: Vec<String> = ron::from_str(include_str!("assets/jwt-rs@jwt-ui.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "arm".to_string(), true);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("jwtui-armv7-musl.tar.gz"));
    }

    #[test]
    fn test_linux_missing_os_label() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/jedisct1@minisign.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("minisign-0.12-linux.tar.gz"));
    }

    #[test]
    fn test_linux_missing_os_label_no_musl_asset_musl_preferred() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/jedisct1@minisign.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86_64".to_string(), true);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("minisign-0.12-linux.tar.gz"));
    }

    #[test]
    fn test_macos_intel_missing_os_label() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/jedisct1@minisign.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("macos".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("minisign-0.12-macos.zip"));
    }

    #[test]
    fn test_macos_arm64_missing_os_label() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/jedisct1@minisign.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("macos".to_string(), "aarch64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("minisign-0.12-macos.zip"));
    }

    #[test]
    fn test_linux_x86_64_arch_label_as_extension() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/vitor-mariano@regex-tui.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("regex-tui_v0.7.0_linux.amd64"));
    }

    #[test]
    fn test_macos_arm64_arch_label_as_extension() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/vitor-mariano@regex-tui.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("macos".to_string(), "aarch64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("regex-tui_v0.7.0_darwin.arm64"));
    }

    #[test]
    fn test_linux_x86_64_os_label_as_extension() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/fantauser@fantarepo_ends_in_os.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("fantarepo_ends_in_os_v1.0.0_amd64.linux"));
    }

    #[test]
    fn test_macos_arm64_os_label_as_extension() {
        let assets: Vec<String> =
            ron::from_str(include_str!("assets/fantauser@fantarepo_ends_in_os.ron")).unwrap();
        let asset_refs: Vec<&str> = assets.iter().map(|s| s.as_str()).collect();
        let platform_triple = AssetTriple::new("macos".to_string(), "aarch64".to_string(), false);
        let binaries = get_triple_compatible_assets(&asset_refs, &platform_triple, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(binaries[0].contains("fantarepo_ends_in_os_v1.0.0_arm64.darwin"));
    }
}
