use reqwest::Client;
use std::fs::File;
use std::path::Path;
use crate::downloader::download_file;

use crate::extract;
use crate::os::OS;

pub async fn download_java(client: &Client, java_install_path: &str, java_path: &str, url: &str, os: &OS) -> Result<(), String> {
    if !Path::new(java_path).exists() {
        if os == &OS::Windows {
            println!("Downloading Java...");
            download_file(client, url, "./java.zip")
                .await
                .expect("Failed to download Java");

            println!("Extracting Java...");
            extract(&File::open("./java.zip").expect("Failed to unzip old Java file"), java_install_path, os);

            println!("Deleting old Java file...");
            std::fs::remove_file("./java.zip").expect("Failed to delete old Java file");
        } else {
            println!("Downloading Java...");
            download_file(client, url, "./java.tar.gz")
                .await
                .expect("Failed to download Java");

            println!("Extracting Java...");
            extract(&File::open("./java.tar.gz").expect("Failed to unzip old Java file"), java_install_path, os);

            println!("Deleting old Java file...");
            std::fs::remove_file("./java.tar.gz").expect("Failed to delete old Java file");
        }
    } else {
        println!("Java is ready.");
    }
    Ok(())
}
