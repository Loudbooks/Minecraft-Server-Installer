use minecraft_server_installer::downloader::download_file;
use tokio::test;
use tempfile::TempDir;
use std::path::Path;

#[tokio::test]
async fn test_download_file_success() {
    let client = reqwest::Client::new();
    let url = "https://google.com/";
    let temp_dir = TempDir::new().expect("Failed to create a temporary directory");
    let temp_file_path = temp_dir.path().join("download");

    tokio::spawn(async move {
        download_file(&client, url, temp_file_path.to_str().unwrap())
            .await
            .expect("Download failed");
    })
    .await
    .expect("Failed to spawn the Tokio task");

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let downloaded_file_path = temp_dir.path().join("download");
    assert!(downloaded_file_path.exists(), "Downloaded file not found");
}
