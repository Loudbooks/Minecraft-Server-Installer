use std::fs::File;
use std::io::Read;
use serde_json::Value;
use tokio::fs;
use xml2json_rs::JsonBuilder;
use crate::downloader::Downloader;

pub(crate) struct NeoForge {}

impl Downloader for NeoForge {
    async fn download(client: reqwest::Client, minecraft_version: Option<String>) -> Result<String, crate::downloaderror::DownloadError> {
        let neo_version = get_neoforge_version(minecraft_version).await.expect("Failed to get latest Neoforge version");

        println!("Using Neoforge version {}.", neo_version);

        let url = format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
            neo_version,
            neo_version
        );

        crate::downloader::download_file(&client, &url, "./neoforge.jar").await?;

        Ok(neo_version)
    }
}

pub async fn build_server(java_path: String, mut minecraft_version: Option<String>) {
    let mut command = std::process::Command::new(java_path.clone());

    if minecraft_version.is_none() {
        minecraft_version = Some(get_neoforge_version(None).await.expect("Failed to get latest forge version"));
    } else {
        minecraft_version = Some(get_neoforge_version(minecraft_version).await.expect("Failed to get latest forge version"));
    }

    let mut process = command
        .arg("-jar")
        .arg("neoforge.jar")
        .arg("--installServer")
        .spawn()
        .expect("Failed to build server");

    println!("Building server with Neoforge version {}. This will take a while...", minecraft_version.clone().unwrap_or("".to_string()));

    process.wait().expect("Failed to build server");

    fs::remove_file("neoforge.jar").await.expect("Failed to remove Neoforge jar");
    fs::rename("run.sh", "launch.sh").await.expect("Failed to rename run.sh to launch.sh");

    let mut content = String::new();

    File::open("launch.sh").expect("Failed to open launch.sh").read_to_string(&mut content).expect("Failed to read launch.sh");
    let new_content = content.replace("java", format!("\"{}\"", java_path.as_str()).as_str());
    fs::write("launch.sh", new_content).await.expect("Failed to write to launch.sh");

    fs::rename("run.bat", "launch.bat").await.expect("Failed to rename run.sh to launch.sh");

    let mut content = String::new();

    File::open("launch.bat").expect("Failed to open launch.bat").read_to_string(&mut content).expect("Failed to read launch.bat");
    let new_content = content.replace("java", format!("\"{}\"", java_path.as_str()).as_str());
    fs::write("launch.bat", new_content).await.expect("Failed to write to launch.bat");

    println!("Server built successfully!");
}

async fn get_version_array() -> Vec<Value> {
    let response = reqwest::get("https://maven.neoforged.net/releases/net/neoforged/neoforge//maven-metadata.xml").await.expect("Failed to get Neoforge metadata");
    let body = response.text().await.expect("Failed to get Neoforge metadata");
    let builder = JsonBuilder::default();
    let json = builder.build_from_xml(body.as_str()).unwrap();

    json.get("metadata").expect("Failed step 1 of Neoforge")
        .get("versioning").expect("Failed step 2 of Neoforge")
        .as_array().expect("Failed step 3 of Neoforge")
        .first().expect("Failed step 4 of Neoforge")
        .get("versions").expect("Failed step 5 of Neoforge")
        .as_array().expect("Failed step 6 of Neoforge")
        .first().expect("Failed step 7 of Neoforge")
        .as_object().expect("Failed step 8 of Neoforge")
        .get("version").expect("Failed step 9 of Neoforge")
        .as_array().expect("Failed step 10 of Neoforge")
        .to_owned()
}

async fn get_neoforge_version(minecraft_version: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
    let versions = get_version_array().await;

    if minecraft_version.is_none() {
        let neoforge_version = versions
            .iter()
            .map(|version| version.as_str().unwrap().to_string())
            .max()
            .expect("Version not found!");

        Ok(neoforge_version)
    } else {
        let cut_version = minecraft_version.unwrap().chars().skip(2).collect::<String>();

        let latest_version = versions
            .iter()
            .filter_map(|version| {
                let version = version.as_str()?;
                if version.starts_with(&cut_version.clone()) {
                    Some(version)
                } else {
                    None
                }
            })
            .max()
            .expect("Version not found!");

        Ok(latest_version.to_string())
    }
}