use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::cmp::min;
use std::fs::File;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use async_trait::async_trait;
use public_ip::addr;
use crate::downloaderror::DownloadError;
use crate::servertype::ServerType;

#[async_trait]
pub trait Installer: Sync {
    fn get_name(&self) -> String;
    fn get_description(&self) -> String;
    fn get_type(&self) -> ServerType;

    async fn startup_message(&self, string: String) -> Option<SocketAddrV4>;
    async fn download(&self, client: Client, minecraft_version: Option<String>) -> Result<String, DownloadError>;
    async fn build(&self, _java_path: String, _minecraft_version: Option<String>) {}
}

pub async fn basic_server_address_from_string(string: String) -> Option<SocketAddrV4> {
    if string.contains("Starting Minecraft server on *:") {
        let parsed_port = string.split("*:").collect::<Vec<&str>>()[1].parse::<u16>().expect("Failed to parse port");
        println!("Port successfully parsed: {}", parsed_port);

        let ipv4: Ipv4Addr = Ipv4Addr::from_str(&*addr().await.expect("Failed to get public IP").to_string()).expect("Failed to parse IP");

        return Some(SocketAddrV4::new(ipv4, parsed_port));
    }

    None
}

pub async fn basic_proxy_address_from_string(string: String) -> Option<SocketAddrV4> {
    if string.contains("Listening on /") {
        let ip = string.split('/').collect::<Vec<&str>>()[1];
        let port = ip.split(':').collect::<Vec<&str>>()[1].parse::<u32>().ok();
        let ip = ip.split(':').collect::<Vec<&str>>()[0];
        let ip = Ipv4Addr::from_str(ip).ok();

        if ip.is_some() && port.is_some() {
            return Some(SocketAddrV4::new(ip.unwrap(), port.unwrap() as u16));
        }
    }

    None
}

pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), DownloadError> {
    let request = client.get(url).send().await?;
    let total_size = request.content_length().unwrap_or(0u64);

    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.green/white}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").expect("Failed to set progress bar style")
        .progress_chars("#>-"));

    let mut file = File::create(path)?;
    let mut download_progress: u64 = 0;
    let mut stream = request.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;

        let new = min(download_progress + (chunk.len() as u64), total_size);

        download_progress = new;
        progress_bar.set_position(new);
    }

    progress_bar.finish_with_message(format!("Downloaded {} to {}.", url, path));

    Ok(())
}

pub async fn version_index(mut minecraft_version: Option<String>) -> Result<i32, DownloadError> {
    let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let manifest_body = reqwest::get(manifest_url).await?.text().await?;
    let manifest_json: serde_json::Value = serde_json::from_str(&manifest_body).expect("Failed to parse manifest JSON");

    if minecraft_version.is_none() {
        minecraft_version = Some(get_latest_vanilla_version().await?);
    }

    let version_array: Vec<&serde_json::Value> = manifest_json
        .get("versions")
        .expect("Failed to get versions")
        .as_array()
        .expect("Failed to get versions as array")
        .iter().rev()
        .collect();

    let version_index = version_array
        .iter()
        .position(|version| version["id"].as_str().expect("Failed to get ID") == minecraft_version.clone().unwrap())
        .expect("Failed to get selected version") as i32;

    Ok(version_index)
}

pub async fn get_latest_vanilla_version() -> Result<String, DownloadError> {
    let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let manifest_body = reqwest::get(manifest_url).await?.text().await?;
    let manifest_json: serde_json::Value = serde_json::from_str(&manifest_body).expect("Failed to parse manifest JSON");

    let latest_version = manifest_json
        .get("latest")
        .expect("Failed to get latest release version")
        .get("release")
        .expect("Failed to get latest release version")
        .as_str()
        .expect("Failed to get latest release version as string")
        .to_string();

    Ok(latest_version)
}