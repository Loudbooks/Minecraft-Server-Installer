
use std::error::Error;
use std::net::SocketAddrV4;
use std::ops::Deref;
use async_trait::async_trait;
use reqwest::Client;
use crate::downloader::{basic_server_address_from_string, download_file, Installer};
use crate::downloaderror::DownloadError;
use crate::servertype::ServerType;
use crate::servertype::ServerType::Server;

pub(crate) struct Purpur {}

#[async_trait]
impl Installer for Purpur {
    fn get_name(&self) -> String {
        "Purpur".to_string()
    }

    fn get_description(&self) -> String {
        "A fast, PaperMC fork.".to_string()
    }

    fn get_type(&self) -> ServerType {
        Server
    }

    fn custom_script(&self) -> bool {
        false
    }

    async fn get_versions(&self, client: Client) -> Vec<String> {
        let json: serde_json::Value = client.get("https://api.purpurmc.org/v2/purpur/").send().await.expect("Failed to get latest version for Purpur").json().await.expect("Failed to get latest version for Purpur");

        let versions = json["versions"].as_array().unwrap();
        let mut version_strings = Vec::new();

        for version in versions {
            let version_string = version.as_str().unwrap().to_string();
            version_strings.push(version_string);
        }

        version_strings
    }

    async fn startup_message(&self, string: String) -> Option<SocketAddrV4> {
        basic_server_address_from_string(string).await
    }

    async fn download(&self, client: Client, minecraft_version: Option<String>) -> Result<String, DownloadError> {
        let purpur_version = get_latest_purpur_version(minecraft_version).await.expect("Failed to get latest Purpur version");
        let latest_build = get_latest_build(&purpur_version).await.expect("Failed to get latest Purpur build");

        println!(
            "Using Purpur version {} with build {}.",
            purpur_version, latest_build
        );

        let url = format!(
            "https://api.purpurmc.org/v2/purpur/{}/{}/download",
            purpur_version,
            latest_build,
        );

        download_file(&client, &url, "./server.jar").await?;

        Ok(purpur_version)
    }
}

async fn get_latest_purpur_version(minecraft_version: Option<String>) -> Result<String, Box<dyn Error>> {
    let url = "https://api.purpurmc.org/v2/purpur/";
    let response = reqwest::get(url).await?;
    let json: serde_json::Value = response.json().await?;
    let versions = json["versions"].as_array().ok_or("JSON is invalid!")?;

    if minecraft_version.is_none() {
        let purpur_version = versions
            .last()
            .and_then(|v| v.as_str())
            .ok_or("Version not found!")?;

        Ok(purpur_version.to_string())
    } else {
        let purpur_version = versions
            .iter()
            .filter_map(|v| v.as_str())
            .find(|v| v.contains(minecraft_version.as_ref().unwrap().deref()))
            .ok_or("Version not found!")?;

        Ok(purpur_version.to_string())
    }
}

async fn get_latest_build(purpur_version: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://api.purpurmc.org/v2/purpur/{}",
        purpur_version
    );
    let response = reqwest::get(&url).await?;
    let json: serde_json::Value = response.json().await?;

    let build = json["builds"]
        .as_object()
        .expect("JSON is invalid")
        .get("latest")
        .expect("No builds found")
        .as_str()
        .expect("Failed to get latest build as string");

    Ok(build.to_string())
}
