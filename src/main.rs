use std::cmp::min;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tar::Archive;

#[tokio::main]
async fn main() {
    let is_arm = env::consts::ARCH.contains("arch64");
    let java_path = if cfg!(target_os = "macos") {
        "./java/jdk-17.0.10+7-jre/Contents/home/bin/java"
    } else if cfg!(target_os = "linux") {
        "./java/jdk-17.0.10+7-jre/bin/java"
    } else {
        "./java/jdk-17.0.10+7-jre/bin/java.exe"
    };

    let client = Client::new();

    if download_java(is_arm, java_path).await.is_err() {
        println!("Failed to download Java.");
        return;
    }

    if File::open("./server.jar").is_ok() {
        println!("A server already exists. Do you want to delete it and download a new one? (y/n)");

        if yes_or_no() {
            std::fs::remove_file("./server.jar").unwrap();
        } else {
            return;
        }
    }

    println!(" ");
    println!("What kind of server do you want to run?");
    println!("1. Vanilla - The original Minecraft server. No plugins or mods.");
    println!("2. Paper - A Minecraft server with plugins.");
    println!("3. FabricMC - A Minecraft server with Fabric mods.");
    println!(" ");
    println!("Enter the number of the server you want to run: (1-3) ");

    let mut server_type = String::new();
    std::io::stdin().read_line(&mut server_type).unwrap();
    server_type = server_type.trim().to_string();

    while server_type.parse::<i32>().is_err() || server_type.parse::<i32>().unwrap() < 1 || server_type.parse::<i32>().unwrap() > 4 {
        println!("Please enter a valid number.");
        server_type = String::new();
        std::io::stdin().read_line(&mut server_type).unwrap();
    }

    if server_type.parse::<i32>().unwrap() == 1 {
        download_vanilla(&client).await.unwrap();
    } else if server_type.parse::<i32>().unwrap() == 2 {
        download_paper(&client).await.unwrap();
    } else if server_type.parse::<i32>().unwrap() == 3 {
        download_fabric(&client).await.unwrap();
    }

    accept_eula().await;

    println!(" ");
    println!("Do you want to create a launch script? (RECOMMENDED FOR NEW USERS) (y/n)");

    if yes_or_no() {
        if cfg!(target_os = "windows") {
            create_launch_bat(java_path).await;
        } else {
            create_launch_sh(java_path).await;
        }
    }

    println!(" ");
    println!("Hava a nice day!");
    println!("Tool was created by Loudbook, contact me on Discord: @loudbook");
    println!(" ");
}

fn yes_or_no() -> bool {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input = input.trim().to_string();

    while input != "y" && input != "n" {
        println!("Please enter y or n.");
        input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }

    return input == "y";
}

async fn create_launch_bat(java_path: &str) {
    println!("Creating launch script...");

    let file = File::create("./launch.bat").unwrap();
    let mut file = std::io::BufWriter::new(file);

    file.write_all(format!("\"{}\" -Xms1024M -Xmx4G -jar server.jar nogui\npause", java_path).as_bytes()).unwrap();

    println!("Launch script was created!");
    println!("To start your server, double click on launch.bat");
}

async fn create_launch_sh(java_path: &str) {
    println!("Creating launch script...");

    let file = File::create("./launch.sh").unwrap();
    let mut file = std::io::BufWriter::new(file);

    file.write_all(format!("\"{}\" -Xms1024M -Xmx4G -jar server.jar nogui\npause", java_path).as_bytes()).unwrap();

    std::process::Command::new("chmod")
        .arg("+x")
        .arg("./launch.sh")
        .output()
        .expect("Failed to chmod launch.sh");

    println!("Launch script was created!");
    println!("To start your server, run ./launch.sh or double click on launch.sh if available.");
}

async fn accept_eula() {
    println!("Checking EULA...");
    let file = File::create("./eula.txt").unwrap();
    let mut file = std::io::BufWriter::new(file);

    file.write_all("eula=true".as_bytes()).unwrap();
}

async fn download_fabric(client: &Client) -> Result<(), String> {
    let fabric_version = get_latest_fabric_version().await.unwrap();
    let fabric_build = get_fabric_build().await.unwrap();

    println!("Using game version {} with Fabric build {}.", fabric_version, fabric_build);

    let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}/{}/1.0.0/server/jar", fabric_version, fabric_build);

    download_file(
        &client,
        &url,
        "./server.jar"
    ).await.unwrap();

    return Ok(());
}

async fn get_latest_fabric_version() -> Result<String, String> {
    let url = "https://meta.fabricmc.net/v2/versions";
    let response = reqwest::get(url).await.unwrap();
    let json: serde_json::Value = response.json().await.unwrap();

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

async fn get_fabric_build() -> Result<String, String> {
    let url = "https://meta.fabricmc.net/v2/versions/loader";
    let response = reqwest::get(url).await.unwrap();
    let json: serde_json::Value = response.json().await.unwrap();

    let stable_fabric_version = json.as_array()
        .and_then(|versions| versions.iter().find(|version| version["stable"].as_bool() == Some(true)))
        .and_then(|version| version["version"].as_str())
        .ok_or("No stable fabric version found")?;

    Ok(stable_fabric_version.to_string())
}


async fn download_paper(client: &Client) -> Result<(), String> {
    let paper_version = get_latest_paper_version().await.unwrap();
    let latest_build = get_latest_build(&paper_version).await.unwrap();

    println!("Using Paper version {} with build {}.", paper_version, latest_build);

    let url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/{}", paper_version, latest_build,
        format!("paper-{}-{}.jar", paper_version, latest_build));

    download_file(
        &client,
        &url,
        "./server.jar"
    ).await.unwrap();

    return Ok(());
}

async fn get_latest_paper_version() -> Result<String, String> {
    let url = "https://papermc.io/api/v2/projects/paper";
    let response = reqwest::get(url).await.unwrap();
    let json: serde_json::Value = response.json().await.unwrap();
    let versions = json["versions"].as_array().ok_or("JSON is invalid!")?;
    let paper_version = versions.last().and_then(|v| v.as_str()).ok_or("Paper version not found!")?;

    return Ok(paper_version.to_string());
}

async fn get_latest_build(paper_version: &str) -> Result<String, String> {
    let url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds", paper_version);
    let response = reqwest::get(&url).await.unwrap();
    let json: serde_json::Value = response.json().await.unwrap();

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

    return Ok(build.to_string())
}

async fn download_vanilla(client: &Client) -> Result<(), String> {
    println!("Downloading Vanilla server...");

    let manifest_url = "https://launchermeta.mojang.com/mc/game/version_manifest.json";
    let manifest_body = reqwest::get(manifest_url).await.unwrap().text().await.unwrap();
    let manifest_json: serde_json::Value = serde_json::from_str(&manifest_body).unwrap();

    let version_url = manifest_json["versions"][1]["url"].as_str().ok_or("Invalid manifest format");
    let version_body = reqwest::get(version_url.unwrap()).await.unwrap().text().await.unwrap();
    let server_url = serde_json::from_str::<serde_json::Value>(&version_body)
        .ok()
        .and_then(|json| {
            let downloads = json.get("downloads")?.clone();
            let server = downloads.get("server")?.clone();
            server.get("url")?.as_str().map(|url| url.to_string())
        });

    download_file(
        &client,
        &server_url.unwrap().to_string(),
        "./server.jar"
    ).await.unwrap();

    return Ok(());
}

async fn download_java(is_arm: bool, java_path: &str) -> Result<(), String> {
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

        let client = Client::new();

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

pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), String> {
    let request = client.get(url).send().await.or(Err("Failed to download file."))?;
    let total_size = request.content_length().unwrap_or(0);

    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap().progress_chars("#>-"));

    let mut file = File::create(path).unwrap();
    let mut download_progress: u64 = 0;
    let mut stream = request.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err("Failed to download file."))?;
        file.write_all(&chunk).or(Err("Failed to write file."))?;

        let new = min(download_progress + (chunk.len() as u64), total_size);

        download_progress = new;
        progress_bar.set_position(new);
    }

    progress_bar.finish_with_message(format!("Downloaded {} to {}.", url, path));

    return Ok(());
}


fn unzip(file: &File) {
    if cfg!(target_os = "windows") {
        let mut archive = zip::ZipArchive::new(file).unwrap();
        archive.extract("./java").unwrap();
    } else {
        let decompressed = GzDecoder::new(file);

        let mut archive = Archive::new(decompressed);
        archive.unpack("./java").unwrap();
    }
}