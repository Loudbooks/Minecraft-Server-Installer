use std::fs::File;
use std::io::Read;
use serde_json::Value;
use tokio::fs;
use xml2json_rs::JsonBuilder;
use crate::downloader::Downloader;
use crate::servertype::ServerType;
use crate::servertype::ServerType::Server;

pub(crate) struct NeoForge {}

impl Downloader for NeoForge {
    fn get_name(&self) -> String {
        "NeoForge".to_string()
    }

    fn get_description(&self) -> String {
        "A server that supports NeoForge mods.".to_string()
    }

    fn get_type(&self) -> ServerType {
        Server
    }

    async fn install(client: reqwest::Client, minecraft_version: Option<String>) -> Result<String, crate::downloaderror::DownloadError> {
        let neo_version = get_neoforge_version(minecraft_version).await.expect("Failed to get latest NeoForge version");

        println!("Using NeoForge version {}.", neo_version);

        let url = format!(
            "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
            neo_version,
            neo_version
        );

        crate::downloader::download_file(&client, &url, "./neoforge.jar").await?;

        Ok(neo_version)
    }

    async fn startup_message(string: &String) -> Option<std::net::SocketAddrV4> {
        crate::downloader::basic_server_address_from_string(string).await
    }
    
    async fn build(java_path: String, mut minecraft_version: Option<String>) {
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

        println!("Building server with NeoForge version {}. This will take a while...", minecraft_version.clone().unwrap_or("".to_string()));

        process.wait().expect("Failed to build server");

        fs::remove_file("neoforge.jar").await.expect("Failed to remove NeoForge jar");
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

        fs::remove_file("user_jvm_args.txt").await.expect("Failed to remove user_jvm_args.txt");

        println!("Server built successfully!");
    }
}

async fn get_version_array() -> Vec<Value> {
    let response = reqwest::get("https://maven.neoforged.net/releases/net/neoforged/neoforge//maven-metadata.xml").await.expect("Failed to get NeoForge metadata");
    let body = response.text().await.expect("Failed to get NeoForge metadata");
    let builder = JsonBuilder::default();
    let json = builder.build_from_xml(body.as_str()).unwrap();

    json.get("metadata").expect("Failed step 1 of NeoForge")
        .get("versioning").expect("Failed step 2 of NeoForge")
        .as_array().expect("Failed step 3 of NeoForge")
        .first().expect("Failed step 4 of NeoForge")
        .get("versions").expect("Failed step 5 of NeoForge")
        .as_array().expect("Failed step 6 of NeoForge")
        .first().expect("Failed step 7 of NeoForge")
        .as_object().expect("Failed step 8 of NeoForge")
        .get("version").expect("Failed step 9 of NeoForge")
        .as_array().expect("Failed step 10 of NeoForge")
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