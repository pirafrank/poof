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
