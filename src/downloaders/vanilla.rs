use reqwest::Client;
use crate::downloader::{download_file, Downloader, get_latest_vanilla_version};
use crate::downloaderror::DownloadError;

pub(crate) struct Vanilla {}

impl Downloader for Vanilla {
    async fn download(client: Client, minecraft_version: Option<String>) -> Result<(), DownloadError> {
        println!("Downloading Vanilla server...");

        let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let manifest_body = reqwest::get(manifest_url).await?.text().await?;
        let manifest_json: serde_json::Value = serde_json::from_str(&manifest_body).expect("Failed to parse manifest JSON");

        let version_number;

        if minecraft_version.is_none() {
            version_number = get_latest_vanilla_version().await?;
        } else {
            version_number = minecraft_version.unwrap();
        }

        println!("Using version {}", version_number);

        let version_url = manifest_json
            .get("versions")
            .expect("Failed to get versions")
            .as_array()
            .expect("Failed to get versions as array")
            .iter()
            .find(|version| version["id"].as_str().expect("Failed to get ID") == version_number)
            .and_then(|version| version["url"].as_str());

        let version_body = reqwest::get(version_url.expect("Version not found!")).await?.text().await?;
        let server_url = serde_json::from_str::<serde_json::Value>(&version_body)
            .ok()
            .and_then(|json| {
                let downloads = json.get("downloads")?.clone();
                let server = downloads.get("server")?.clone();
                server.get("url")?.as_str().map(|url| url.to_string())
            }).expect("Failed to get server URL");

        download_file(&client, &server_url.to_string(), "./server.jar").await?;

        return Ok(());
    }
}
