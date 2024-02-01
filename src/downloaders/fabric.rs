use reqwest::Client;
use std::error::Error;
use crate::downloader::{download_file, Downloader};
use crate::downloaderror::DownloadError;

pub(crate) struct Fabric {}

impl Downloader for Fabric {
    async fn download(client: Client, minecraft_version: Option<String>) -> Result<String, DownloadError> {
        let fabric_version = get_latest_fabric_version(&minecraft_version).await.expect("Failed to get latest fabric version");
        let fabric_build = get_fabric_build().await.expect("Failed to get latest fabric build");

        println!(
            "Using game version {} with Fabric build {}.",
            fabric_version, fabric_build
        );

        let url = format!(
            "https://meta.fabricmc.net/v2/versions/loader/{}/{}/1.0.0/server/jar",
            fabric_version, fabric_build
        );

        download_file(&client, &url, "./server.jar").await?;

        Ok(fabric_version.to_string())
    }
}

async fn get_latest_fabric_version(minecraft_version: &Option<String>) -> Result<String, Box<dyn Error>> {
    let url = "https://meta.fabricmc.net/v2/versions";
    let response = reqwest::get(url).await?;
    let json: serde_json::Value = response.json().await?;

    let game_versions = json["game"].as_array().ok_or("Invalid JSON format")?;

    if minecraft_version.is_none() {
        let stable_game_version = game_versions
            .iter()
            .filter_map(|version| {
                let is_stable = version["stable"].as_bool()?;
                if is_stable {
                    version["version"].as_str().map(|v| v.to_string())
                } else {
                    None
                }
            })
            .max()
            .ok_or("No stable game version found")?;

        Ok(stable_game_version)
    } else {
        let stable_game_version = game_versions
            .iter()
            .filter_map(|version| {
                let version = version["version"].as_str()?;
                if version.eq(minecraft_version.clone().unwrap().as_str()) {
                    Some(version)
                } else {
                    None
                }
            })
            .max()
            .ok_or("Version not found!")?;

        Ok(stable_game_version.to_string())
    }
}

async fn get_fabric_build() -> Result<String, Box<dyn Error>> {
    let url = "https://meta.fabricmc.net/v2/versions/loader";
    let response = reqwest::get(url).await?;
    let json: serde_json::Value = response.json().await?;

    let stable_fabric_version = json
        .as_array()
        .and_then(|versions| {
            versions
                .iter()
                .find(|version| version["stable"].as_bool() == Some(true))
        })
        .and_then(|version| version["version"].as_str())
        .ok_or("No stable fabric version found")?;

    Ok(stable_fabric_version.to_string())
}
