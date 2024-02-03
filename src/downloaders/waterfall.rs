
use std::error::Error;
use std::net::SocketAddrV4;
use async_trait::async_trait;
use reqwest::Client;
use crate::downloader::{basic_server_address_from_string, download_file, Installer};
use crate::downloaderror::DownloadError;
use crate::servertype::ServerType;
use crate::servertype::ServerType::Proxy;

pub(crate) struct Waterfall {}

#[async_trait]
impl Installer for Waterfall {
    fn get_name(&self) -> String {
        "Waterfall".to_string()
    }

    fn get_description(&self) -> String {
        "A proxy that supports Bungeecord plugins, by PaperMC".to_string()
    }

    fn get_type(&self) -> ServerType {
        Proxy
    }

    fn custom_script(&self) -> bool {
        false
    }

    async fn startup_message(&self, string: String) -> Option<SocketAddrV4> {
        basic_server_address_from_string(string).await
    }

    async fn download(&self, client: Client, minecraft_version: Option<String>) -> Result<String, DownloadError> {
        let waterfall_version = get_latest_waterfall_version(minecraft_version).await.expect("Failed to get latest waterfall version");
        let latest_build = get_latest_build(&waterfall_version).await.expect("Failed to get latest waterfall build");

        println!(
            "Using Waterfall version {} with build {}.",
            waterfall_version, latest_build
        );

        let url = format!(
            "https://api.papermc.io/v2/projects/waterfall/versions/{}/builds/{}/downloads/waterfall-{}-{}.jar",
            waterfall_version,
            latest_build,
            waterfall_version, latest_build
        );

        download_file(&client, &url, "./server.jar").await?;

        Ok(waterfall_version)
    }
}

async fn get_latest_waterfall_version(minecraft_version: Option<String>) -> Result<String, Box<dyn Error>> {
    let url = "https://papermc.io/api/v2/projects/waterfall";
    let response = reqwest::get(url).await?;
    let json: serde_json::Value = response.json().await?;
    let versions = json["versions"].as_array().ok_or("JSON is invalid!")?;

    if minecraft_version.is_none() {
        let waterfall_version = versions
            .last()
            .and_then(|v| v.as_str())
            .ok_or("Version not found!")?;

        Ok(waterfall_version.to_string())
    } else {
        let minecraft_version = minecraft_version.unwrap();
        let waterfall_version = versions
            .iter()
            .filter_map(|version| {
                let version = version.as_str()?;
                if version.starts_with(&minecraft_version) {
                    Some(version)
                } else {
                    None
                }
            })
            .max()
            .ok_or("Version not found!")?;

        Ok(waterfall_version.to_string())
    }
}

async fn get_latest_build(waterfall_version: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://api.papermc.io/v2/projects/waterfall/versions/{}/builds",
        waterfall_version
    );
    let response = reqwest::get(&url).await?;
    let json: serde_json::Value = response.json().await?;

    let build = json["builds"]
        .as_array()
        .ok_or("JSON is invalid")?
        .iter()
        .filter_map(|build| {
            let channel = build["channel"].as_str()?;
            let build_number = build["build"].as_u64()?;
            if channel == "default" {
                Some(build_number)
            } else {
                None
            }
        })
        .max()
        .ok_or("No builds found")?;

    Ok(build.to_string())
}
