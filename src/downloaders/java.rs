use std::fs::File;
use std::path::Path;
use reqwest::Client;
use crate::downloaders::downloader::download_file;

use crate::unzip;

pub async fn download(client: &Client, is_arm: bool, java_path: &str) -> Result<(), String> {
    if !Path::new(java_path).exists() {
        let url: Option<&str>;
        #[cfg(target_os = "windows")] { url = Some("https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_windows_hotspot_17.0.10_7.zip") }

        if is_arm {
            #[cfg(target_os = "macos")] { url = Some("https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_aarch64_mac_hotspot_17.0.10_7.tar.gz") }
            #[cfg(target_os = "linux")] { url = Some("https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_aarch64_linux_hotspot_17.0.10_7.tar.gz") }
        } else {
            #[cfg(target_os = "macos")] { url = Some("https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_mac_hotspot_17.0.10_7.tar.gz") }
            #[cfg(target_os = "linux")] { url = Some("https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_linux_hotspot_17.0.10_7.tar.gz") }
        }

        if url.is_none() {
            println!("Unsupported OS.");
            return Ok(());
        }

        if cfg!(target_os = "windows") {
            println!("Downloading Java...");
            download_file(&client, &url.unwrap().to_string(), "./java.zip").await.expect("Failed to download Java.");

            println!("Unzipping Java...");
            unzip(&File::open("./java.zip").expect("Failed to unzip old Java file."));

            println!("Deleting old Java file...");
            std::fs::remove_file("./java.zip").expect("Failed to delete old Java file.");
        } else {
            println!("Downloading Java...");
            download_file(&client, &url.unwrap().to_string(), "./java.tar.gz").await.expect("Failed to download Java.");

            println!("Unzipping Java...");
            unzip(&File::open("./java.tar.gz").expect("Failed to unzip old Java file."));

            println!("Deleting old Java file...");
            std::fs::remove_file("./java.tar.gz").expect("Failed to delete old Java file.");
        }
    }

    return Ok(());
}