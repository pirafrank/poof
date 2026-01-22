use super::*;
use mockito::Server;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_download_asset_success() -> Result<()> {
    let mut server = Server::new();
    let body = "fake binary content";
    let _m = server
        .mock("GET", "/test-asset")
        .with_status(200)
        .with_body(body)
        .create();

    let tmp_dir = tempdir()?;
    let download_to = tmp_dir.path().to_path_buf();
    let filename = "test-file.bin".to_string();
    let download_url = format!("{}/test-asset", server.url());

    let result = download_asset(&filename, &download_url, &download_to)?;

    assert!(result.exists());
    assert_eq!(result, download_to.join(&filename));
    assert_eq!(fs::read_to_string(result)?, body);

    Ok(())
}

#[test]
fn test_download_asset_http_error() -> Result<()> {
    let mut server = Server::new();
    let _m = server
        .mock("GET", "/error-asset")
        .with_status(404)
        .with_body("Not Found")
        .create();

    let tmp_dir = tempdir()?;
    let download_to = tmp_dir.path().to_path_buf();
    let filename = "error-file.bin".to_string();
    let download_url = format!("{}/error-asset", server.url());

    let result = download_asset(&filename, &download_url, &download_to);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Download failed!"));
    assert!(err_msg.contains("404 Not Found"));
    assert!(err_msg.contains("Not Found"));

    Ok(())
}

#[test]
fn test_download_asset_invalid_url() {
    let tmp_dir = tempdir().unwrap();
    let download_to = tmp_dir.path().to_path_buf();
    let filename = "invalid-url.bin".to_string();
    let download_url = "http://invalid.url.that.does.not.exist.local".to_string();

    let result = download_asset(&filename, &download_url, &download_to);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to initiate download"));
}

#[test]
#[cfg(unix)]
fn test_download_asset_fs_error() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut server = Server::new();
    let _m = server
        .mock("GET", "/test-asset")
        .with_status(200)
        .with_body("content")
        .create();

    let tmp_dir = tempdir()?;
    let download_to = tmp_dir.path().join("no-permission");
    fs::create_dir_all(&download_to)?;

    // Set directory to read-only to trigger failure when creating file inside it
    let mut perms = fs::metadata(&download_to)?.permissions();
    perms.set_mode(0o555); // Read and execute only
    fs::set_permissions(&download_to, perms.clone())?;

    let filename = "test-file.bin".to_string();
    let download_url = format!("{}/test-asset", server.url());

    let result = download_asset(&filename, &download_url, &download_to);

    // Cleanup permissions so tempdir can be deleted
    perms.set_mode(0o755);
    let _ = fs::set_permissions(&download_to, perms);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to create file"));

    Ok(())
}
