use std::net::SocketAddrV4;
use async_trait::async_trait;
use reqwest::Client;
use crate::downloader::{basic_server_address_from_string, download_file, Installer, get_latest_vanilla_version};
use crate::downloaderror::DownloadError;
use crate::servertype::ServerType;
use crate::servertype::ServerType::Server;

pub(crate) struct Vanilla {}

#[async_trait]
impl Installer for Vanilla {
    fn get_name(&self) -> String {
        "Vanilla".to_string()
    }

    fn get_description(&self) -> String {
        "A basic Vanilla server.".to_string()
    }

    fn get_type(&self) -> ServerType {
        Server
    }

    fn custom_script(&self) -> bool {
        false
    }

    async fn get_versions(&self, _client: Client) -> Vec<String> {
        vec!["All Versions".to_string()] // Literally all versions. This is a Vanilla server, after all.
    }

    async fn startup_message(&self, string: String) -> Option<SocketAddrV4> {
        basic_server_address_from_string(string).await
    }

    async fn download(&self, client: Client, minecraft_version: Option<String>) -> Result<String, DownloadError> {
        let mut minecraft_version = minecraft_version;

        println!("Downloading Vanilla server...");

        let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
        let manifest_body = reqwest::get(manifest_url).await?.text().await?;
        let manifest_json: serde_json::Value = serde_json::from_str(&manifest_body).expect("Failed to parse manifest JSON");

        if minecraft_version.is_none() {
            minecraft_version = Some(get_latest_vanilla_version().await?);
        }

        println!("Using version {}", &minecraft_version.as_ref().unwrap());

        let version_url = manifest_json
            .get("versions")
            .expect("Failed to get versions")
            .as_array()
            .expect("Failed to get versions as array")
            .iter()
            .find(|version| version["id"].as_str().expect("Failed to get ID") == minecraft_version.clone().unwrap())
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

        Ok(minecraft_version.unwrap().to_string())
    }
}
