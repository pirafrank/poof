use crate::files::utils::clean_up_filename;

#[test]
fn test_clean_up_filename() {
    let filename = "sometool-1.0.0-linux-x86_64";
    let result = clean_up_filename(
        filename,
        vec![
            "1.0.0".to_string(),
            "linux".to_string(),
            "x86_64".to_string(),
        ],
    );
    assert_eq!(result, "sometool");
}

#[test]
fn test_clean_up_filename_with_middlename_separator() {
    let filename = "some-tool-1.0.0-linux-x86_64";
    let result = clean_up_filename(
        filename,
        vec![
            "1.0.0".to_string(),
            "linux".to_string(),
            "x86_64".to_string(),
        ],
    );
    assert_eq!(result, "some-tool");
}

#[test]
fn test_clean_up_filename_with_os_in_name() {
    let filename = "linuxtool-1.0.0-linux-x86_64";
    let result = clean_up_filename(
        filename,
        vec![
            "1.0.0".to_string(),
            "linux".to_string(),
            "x86_64".to_string(),
        ],
    );
    assert_eq!(result, "linuxtool");
}

#[test]
fn cleanup_preserves_variants_and_collapses_mixed_separators() {
    let tokens = vec![
        "x86".into(),
        "x86_64".into(),
        "linux".into(),
        "v1.2.3".into(),
    ];
    assert_eq!(
        clean_up_filename("linuxtool_v1.2.3-LINUX--_x86_64__pkcs11key", tokens),
        "linuxtool_pkcs11key"
    );
}

#[test]
fn cleanup_preserves_untouched_names_and_punctuation() {
    let tokens = vec![
        "linux".into(),
        "amd64".into(),
        "pc".into(),
        "apple".into(),
        "gnu".into(),
    ];
    for name in ["python3.13", "clang++", "café", "toolname", "tool_name"] {
        assert_eq!(clean_up_filename(name, tokens.clone()), name);
    }
    assert_eq!(
        clean_up_filename("clang++-linux-amd64", tokens.clone()),
        "clang++"
    );
    assert_eq!(
        clean_up_filename("tool+linux", tokens.clone()),
        "tool+linux"
    );
    assert_eq!(clean_up_filename("café-linux-amd64", tokens), "café");
    // Counter-example of asset_names_prioritize_the_first_dot_separated_component:
    // First-dot truncation belongs to asset normalization,
    // not shared token cleanup applied to executable filenames, as well.
    // This is needed to demonstrate that cleanup function works, albeit it's
    // kind of rare that a filename would have a dot after the shared token.
    assert_eq!(
        clean_up_filename("tool-linux.helper", vec!["linux".into(), String::new()]),
        "tool-helper"
    );
}
