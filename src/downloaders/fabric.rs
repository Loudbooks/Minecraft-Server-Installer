use std::error::Error;
use reqwest::Client;
use crate::downloaders::downloader::{download_file, Downloader};
use crate::downloaders::downloaderror::DownloadError;

pub(crate) struct Fabric {}

impl Downloader for Fabric {
    async fn download(client: &Client) -> Result<(), DownloadError> {
        let fabric_version = get_latest_fabric_version().await.unwrap();
        let fabric_build = get_fabric_build().await.unwrap();

        println!("Using game version {} with Fabric build {}.", fabric_version, fabric_build);

        let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}/{}/1.0.0/server/jar", fabric_version, fabric_build);

        download_file(
            &client,
            &url,
            "./server.jar"
        ).await?;

        return Ok(());
    }
}

async fn get_latest_fabric_version() -> Result<String, Box<dyn Error>> {
    let url = "https://meta.fabricmc.net/v2/versions";
    let response = reqwest::get(url).await?;
    let json: serde_json::Value = response.json().await?;

    let game_versions = json["game"]
        .as_array()
        .ok_or("Invalid JSON format")?;

    let stable_game_version = game_versions.iter()
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

    return Ok(stable_game_version)
}

async fn get_fabric_build() -> Result<String, Box<dyn Error>> {
    let url = "https://meta.fabricmc.net/v2/versions/loader";
    let response = reqwest::get(url).await?;
    let json: serde_json::Value = response.json().await?;

    let stable_fabric_version = json.as_array()
        .and_then(|versions| versions.iter().find(|version| version["stable"].as_bool() == Some(true)))
        .and_then(|version| version["version"].as_str())
        .ok_or("No stable fabric version found")?;

    Ok(stable_fabric_version.to_string())
}

