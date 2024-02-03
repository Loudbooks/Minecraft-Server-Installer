
use std::error::Error;
use std::net::SocketAddrV4;
use async_trait::async_trait;
use reqwest::Client;
use crate::downloader::{basic_proxy_address_from_string, download_file, Installer};
use crate::downloaderror::DownloadError;
use crate::servertype::ServerType;
use crate::servertype::ServerType::Proxy;

pub(crate) struct Velocity {}

#[async_trait]
impl Installer for Velocity {
    fn get_name(&self) -> String {
        "Velocity".to_string()
    }

    fn get_description(&self) -> String {
        "A proxy that supports Velocity plugins.".to_string()
    }

    fn get_type(&self) -> ServerType {
        Proxy
    }

    fn custom_script(&self) -> bool {
        false
    }

    fn version_required(&self) -> bool {
        false
    }

    async fn startup_message(&self, string: String) -> Option<SocketAddrV4> {
        basic_proxy_address_from_string(string).await
    }

    async fn download(&self, client: Client, _minecraft_version: Option<String>) -> Result<String, DownloadError> {
        let velocity_version = get_latest_velocity_version().await.expect("Failed to get latest velocity version");
        let latest_build = get_latest_build(&velocity_version).await.expect("Failed to get latest velocity build");

        println!(
            "Using Velocity version {} with build {}.",
            velocity_version, latest_build
        );

        let url = format!(
            "https://api.papermc.io/v2/projects/velocity/versions/{}/builds/{}/downloads/velocity-{}-{}.jar",
            velocity_version,
            latest_build,
            velocity_version, latest_build
        );

        download_file(&client, &url, "./server.jar").await?;

        Ok(velocity_version)
    }
}

async fn get_latest_velocity_version() -> Result<String, Box<dyn Error>> {
    let url = "https://papermc.io/api/v2/projects/velocity";
    let response = reqwest::get(url).await?;
    let json: serde_json::Value = response.json().await?;
    let versions = json["versions"].as_array().ok_or("JSON is invalid!")?;

    let velocity_version = versions
        .last()
        .and_then(|v| v.as_str())
        .ok_or("Version not found!")?;

    Ok(velocity_version.to_string())
}

async fn get_latest_build(velocity_version: &str) -> Result<String, Box<dyn Error>> {
    let url = format!(
        "https://api.papermc.io/v2/projects/velocity/versions/{}/builds",
        velocity_version
    );
    let response = reqwest::get(&url).await?;
    let json: serde_json::Value = response.json().await?;

    let build = json["builds"]
        .as_array()
        .ok_or("JSON is invalid")?
        .iter()
        .max_by_key(|v| v.get("build").unwrap().as_u64().unwrap())
        .unwrap()
        .get("build")
        .ok_or("No builds found")?;

    Ok(build.to_string())
}
