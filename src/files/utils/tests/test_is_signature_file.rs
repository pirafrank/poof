use crate::files::utils::is_signature_file;

#[test]
fn test_is_signature_file_asc() {
    assert!(is_signature_file("tool-1.0.tar.gz.asc"));
}

#[test]
fn test_is_signature_file_sig() {
    assert!(is_signature_file("tool-1.0.tar.gz.sig"));
}

#[test]
fn test_is_signature_file_pem() {
    assert!(is_signature_file("tool.pem"));
}

#[test]
fn test_is_signature_file_minisign() {
    assert!(is_signature_file("tool-1.0.tar.gz.minisign"));
}

#[test]
fn test_is_signature_file_pgp() {
    assert!(is_signature_file("tool-1.0.tar.gz.pgp"));
}

#[test]
fn test_is_signature_file_gpg() {
    assert!(is_signature_file("tool-1.0.tar.gz.gpg"));
}

#[test]
fn test_is_signature_file_case_insensitive() {
    assert!(is_signature_file("TOOL.ASC"));
}

#[test]
fn test_is_signature_file_non_matching() {
    assert!(!is_signature_file("tool-1.0.tar.gz"));
}
