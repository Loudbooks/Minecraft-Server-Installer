use std::error::Error;
use std::fs;
use std::fs::File;
use std::process::Command;
use reqwest::Client;
use semver::Version;
use serde_json::Value;
use crate::downloader::Downloader;
use crate::downloaderror::DownloadError;

pub(crate) struct Forge {}

impl Downloader for Forge {
    async fn download(client: Client, mut minecraft_version: Option<String>) -> Result<String, DownloadError> {
        let forge_version = get_forge_build(minecraft_version.clone()).await.expect("Failed to get latest forge version");

        if minecraft_version.is_none() {
            minecraft_version = Some(get_latest_forge_version().await.expect("Failed to get latest forge version"));
        }

        println!(
            "Using game version {} with Forge version {}.",
            minecraft_version.clone().unwrap_or("".to_string()),
            forge_version
        );

        let url = if minecraft_version.clone().unwrap().split('.').collect::<Vec<&str>>().get(1).unwrap().eq(&"7") {
            format!(
                "https://files.minecraftforge.net/maven/net/minecraftforge/forge/{}-{}-{}/forge-{}-{}-{}-installer.jar",
                minecraft_version.clone().unwrap_or("".to_string()),
                forge_version,
                minecraft_version.clone().unwrap_or("".to_string()),
                minecraft_version.clone().unwrap_or("".to_string()),
                forge_version,
                minecraft_version.clone().unwrap_or("".to_string()),
            )
        } else {
            format!(
                "https://files.minecraftforge.net/maven/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
                minecraft_version.clone().unwrap_or("".to_string()),
                forge_version,
                minecraft_version.clone().unwrap_or("".to_string()),
                forge_version
            )
        };

        crate::downloader::download_file(&client, &url, "./forge.jar").await?;

        Ok(minecraft_version.clone().expect("Failed to get minecraft version").to_string())
    }
}

pub async fn build_server(java_path: String, mut minecraft_version: Option<String>) {
    let mut command = Command::new(java_path);

    if minecraft_version.is_none() {
        minecraft_version = Some(get_latest_forge_version().await.expect("Failed to get latest forge version"));
    }

    let mut process = command
        .arg("-jar")
        .arg("forge.jar")
        .arg("--installServer")
        .arg(".")
        .spawn()
        .expect("Failed to build server");

    println!("Building server with Forge version {}. This will take a while...", minecraft_version.clone().unwrap_or("".to_string()));

    process.wait().expect("Failed to build server");

    let forge_version = get_forge_build(minecraft_version.clone()).await.expect("Failed to get latest forge version");
    let mut file_name = format!("./forge-{}-{}-shim.jar", minecraft_version.clone().unwrap(), forge_version);

    if File::open(&file_name).is_err() {
        file_name = format!("./minecraftforge-universal-{}-{}.jar", minecraft_version.clone().unwrap(), forge_version);
    }
    if File::open(&file_name).is_err() {
        file_name = format!("./forge-{}-{}.jar", minecraft_version.unwrap(), forge_version);
    }

    fs::rename(format!("./{}", file_name), "./server.jar").expect("Failed to rename server file");
    fs::remove_file("./forge.jar").expect("Failed to delete forge file");
}

async fn get_forge_build(minecraft_version: Option<String>) -> Result<String, Box<dyn Error>> {
    let url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
    let response = reqwest::get(url).await?;
    let json: Value = response.json().await?;

    let game_versions = json["promos"].as_object().ok_or("Invalid JSON format")?;

    if minecraft_version.is_none() {
        let max_version = game_versions
            .iter()
            .flat_map(|(_, v)| v.as_str())
            .flat_map(|v| Version::parse(v).ok())
            .max();

        Ok(max_version.unwrap().to_string())
    } else {
        let version = game_versions
            .iter()
            .filter_map(|(version, build)| {
                let version = version.to_string().replace("-latest", "").replace("-recommended", "");
                if version.to_string().eq(minecraft_version.clone().unwrap().as_str()) {
                    build.as_str().map(|v| v.to_string())
                } else {
                    None
                }
            })
            .max()
            .ok_or("No forge version found")?;

        Ok(version)
    }
}

async fn get_latest_forge_version() -> Result<String, Box<dyn Error>> {
    let url = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
    let response = reqwest::get(url).await?;
    let json: Value = response.json().await?;

    let game_versions = json["promos"].as_object().ok_or("Invalid JSON format")?;

    let max_version = game_versions
        .iter()
        .flat_map(|(_, v)| v.as_str())
        .flat_map(|v| Version::parse(v).ok())
        .max();

    let minecraft_version = game_versions
        .iter()
        .filter_map(|(version, build)| {
            let version = version.to_string().replace("-latest", "").replace("-recommended", "");
            if build.to_string().replace('"', "").eq(max_version.clone().unwrap().to_string().as_str()) {
                Some(version)
            } else {
                None
            }
        })
        .max();

    Ok(minecraft_version.unwrap().to_string())
}