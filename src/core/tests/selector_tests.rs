#[cfg(test)]
mod selector_tests {

    use crate::core::selector::get_triple_compatible_assets;
    use crate::models::asset_triple::AssetTriple;

    #[test]
    fn test_linux_x86_64_non_musl() {
        // Linux x86_64 non-musl
        let assets = vec![
            "checksums.txt",
            "lazygit_0.58.1_darwin_arm64.tar.gz",
            "lazygit_0.58.1_darwin_x86_64.tar.gz",
            "lazygit_0.58.1_freebsd_32-bit.tar.gz",
            "lazygit_0.58.1_freebsd_arm64.tar.gz",
            "lazygit_0.58.1_freebsd_armv6.tar.gz",
            "lazygit_0.58.1_freebsd_x86_64.tar.gz",
            "lazygit_0.58.1_linux_32-bit.tar.gz",
            "lazygit_0.58.1_linux_arm64.tar.gz",
            "lazygit_0.58.1_linux_armv6.tar.gz",
            "lazygit_0.58.1_linux_x86_64.tar.gz",
            "lazygit_0.58.1_windows_32-bit.zip",
            "lazygit_0.58.1_windows_arm64.zip",
            "lazygit_0.58.1_windows_armv6.zip",
            "lazygit_0.58.1_windows_x86_64.zip",
        ];
        let triple_linux_x64 = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&assets, &triple_linux_x64, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries.contains(&"lazygit_0.58.1_linux_x86_64.tar.gz"));
    }

    #[test]
    fn test_linux_x86_64_musl() {
        // Linux x86_64 musl
        let assets = vec![
            "lsd-musl_1.2.0_amd64.deb",
            "lsd-musl_1.2.0_amd64_xz.deb",
            "lsd-musl_1.2.0_arm64.deb",
            "lsd-musl_1.2.0_arm64_xz.deb",
            "lsd-musl_1.2.0_i686.deb",
            "lsd-musl_1.2.0_i686_xz.deb",
            "lsd-v1.2.0-aarch64-apple-darwin.tar.gz",
            "lsd-v1.2.0-aarch64-unknown-linux-gnu.tar.gz",
            "lsd-v1.2.0-aarch64-unknown-linux-musl.tar.gz",
            "lsd-v1.2.0-arm-unknown-linux-gnueabihf.tar.gz",
            "lsd-v1.2.0-i686-pc-windows-gnu.zip",
            "lsd-v1.2.0-i686-pc-windows-msvc.zip",
            "lsd-v1.2.0-i686-unknown-linux-gnu.tar.gz",
            "lsd-v1.2.0-i686-unknown-linux-musl.tar.gz",
            "lsd-v1.2.0-x86_64-apple-darwin.tar.gz",
            "lsd-v1.2.0-x86_64-pc-windows-gnu.zip",
            "lsd-v1.2.0-x86_64-pc-windows-msvc.zip",
            "lsd-v1.2.0-x86_64-unknown-linux-gnu.tar.gz",
            "lsd-v1.2.0-x86_64-unknown-linux-musl.tar.gz",
            "lsd_1.2.0_amd64.deb",
            "lsd_1.2.0_amd64_xz.deb",
            "lsd_1.2.0_arm64.deb",
            "lsd_1.2.0_arm64_xz.deb",
            "lsd_1.2.0_i686.deb",
            "lsd_1.2.0_i686_xz.deb",
        ];
        let triple_linux_x64 = AssetTriple::new("linux".to_string(), "x86_64".to_string(), true);
        let binaries = get_triple_compatible_assets(&assets, &triple_linux_x64, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries.contains(&"lsd-v1.2.0-x86_64-unknown-linux-musl.tar.gz"));
    }

    #[test]
    fn test_linux_i686_non_musl() {
        let assets = vec![
            "fd-musl_10.3.0_amd64.deb",
            "fd-musl_10.3.0_arm64.deb",
            "fd-musl_10.3.0_armhf.deb",
            "fd-musl_10.3.0_i686.deb",
            "fd-v10.3.0-aarch64-apple-darwin.tar.gz",
            "fd-v10.3.0-aarch64-pc-windows-msvc.zip",
            "fd-v10.3.0-aarch64-unknown-linux-gnu.tar.gz",
            "fd-v10.3.0-aarch64-unknown-linux-musl.tar.gz",
            "fd-v10.3.0-arm-unknown-linux-gnueabihf.tar.gz",
            "fd-v10.3.0-arm-unknown-linux-musleabihf.tar.gz",
            "fd-v10.3.0-i686-pc-windows-msvc.zip",
            "fd-v10.3.0-i686-unknown-linux-gnu.tar.gz",
            "fd-v10.3.0-i686-unknown-linux-musl.tar.gz",
            "fd-v10.3.0-x86_64-apple-darwin.tar.gz",
            "fd-v10.3.0-x86_64-pc-windows-gnu.zip",
            "fd-v10.3.0-x86_64-pc-windows-msvc.zip",
            "fd-v10.3.0-x86_64-unknown-linux-gnu.tar.gz",
            "fd-v10.3.0-x86_64-unknown-linux-musl.tar.gz",
            "fd_10.3.0_amd64.deb",
            "fd_10.3.0_arm64.deb",
            "fd_10.3.0_armhf.deb",
            "fd_10.3.0_i686.deb",
        ];
        let triple_linux_i686 = AssetTriple::new("linux".to_string(), "x86".to_string(), false);
        let binaries = get_triple_compatible_assets(&assets, &triple_linux_i686, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries[0].contains(&"fd-v10.3.0-i686-unknown-linux-gnu.tar.gz"));
    }

    #[test]
    fn test_linux_x86_64_multiple_compatible_binaries_without_extension() {
        let assets = vec![
            "tabiew-aarch64-unknown-linux-gnu.deb",
            "tabiew-aarch64-unknown-linux-gnu.rpm",
            "tabiew-armv7-unknown-linux-gnueabihf.deb",
            "tabiew-armv7-unknown-linux-gnueabihf.rpm",
            "tabiew-x86_64-unknown-linux-gnu.deb",
            "tabiew-x86_64-unknown-linux-gnu.rpm",
            "tw-aarch64-apple-darwin",
            "tw-aarch64-pc-windows-msvc.exe",
            "tw-aarch64-unknown-linux-gnu",
            "tw-armv7-unknown-linux-gnueabihf",
            "tw-x86_64-apple-darwin",
            "tw-x86_64-pc-windows-msvc.exe",
            "tw-x86_64-unknown-linux-gnu",
        ];
        let triple_linux_x64 = AssetTriple::new("linux".to_string(), "x86_64".to_string(), false);
        let binaries = get_triple_compatible_assets(&assets, &triple_linux_x64, |asset| asset);
        assert!(binaries.is_some());
        let binaries = binaries.unwrap();
        assert!(!binaries.is_empty() && binaries.len() == 1);
        assert!(binaries.contains(&"tw-x86_64-unknown-linux-gnu"));
    }
}
