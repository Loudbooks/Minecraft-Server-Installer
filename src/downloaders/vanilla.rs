use reqwest::Client;
use crate::downloaders::downloader::{download_file, Downloader};
use crate::downloaders::downloaderror::DownloadError;

pub(crate) struct Vanilla {}

impl Downloader for Vanilla {
    async fn download(client: Client) -> Result<(), DownloadError> {
        println!("Downloading Vanilla server...");

        let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let manifest_body = reqwest::get(manifest_url).await?.text().await?;
        let manifest_json: serde_json::Value = serde_json::from_str(&manifest_body).unwrap();

        let version_url = manifest_json["versions"][1]["url"].as_str().ok_or("Invalid manifest format");
        let version_body = reqwest::get(version_url.unwrap()).await?.text().await?;
        let server_url = serde_json::from_str::<serde_json::Value>(&version_body)
            .ok()
            .and_then(|json| {
                let downloads = json.get("downloads")?.clone();
                let server = downloads.get("server")?.clone();
                server.get("url")?.as_str().map(|url| url.to_string())
            });

        download_file(
            &client,
            &server_url.unwrap().to_string(),
            "./server.jar"
        ).await?;

        return Ok(());
    }
}