extern crate core;

pub mod downloaders;
pub mod config;
pub mod downloader;
pub mod downloaderror;

use flate2::read::GzDecoder;
use reqwest::Client;
use std::env;
use std::fs::File;
use std::io::Write;
use tar::Archive;
use crate::downloader::Downloader;
use crate::downloaders::fabric::Fabric;
use crate::downloaders::java::download_java;
use crate::downloaders::paper::Paper;
use crate::downloaders::vanilla::Vanilla;

#[tokio::main]
async fn main() {
    let is_arm = env::consts::ARCH.contains("arch64") || env::consts::ARCH.contains("arm");

    let os = if cfg!(target_os = "macos") {
        "osx"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        println!("Unsupported OS.");
        return;
    };

    let config = config::ConfigFile {
        path: if os == "windows" {
            env::var("APPDATA").expect("Failed to retrieve APPDATA variable.") + "\\MinecraftServerInstaller"
        } else if os == "linux" {
            env::var("XDG_CONFIG_HOME").expect("Failed to retrieve XDG_CONFIG_HOME variable.") + "/MinecraftServerInstaller"
        } else {
            env::var("HOME").expect("Failed to retrieve HOME variable.") + "/Library/Application Support/MinecraftServerInstaller"
        }
    };

    if !std::path::Path::new(&(config.path.to_string() + "/msi-config.toml")).exists() {
        if !std::path::Path::new(&config.path).exists() {
            std::fs::create_dir(&config.path).expect("Failed to create config directory.");
        }

        config.create();
    }

    let mut ready = false;

    if os == "windows" {
        if File::open("./launch.bat").is_ok() {
            ready = true;
        }
    } else {
        if File::open("./launch.sh").is_ok() {
            ready = true;
        }
    }

    if File::open("./server.jar").is_ok() {
        ready = true;
    }

    if ready {
        println!();
        println!("A server is already set up. What do you want to do?");
        println!("1. Change amount of RAM allocated to the server.");
        println!("2. Replace your server file.");
        println!("3. Exit.");
        println!();
        println!("Enter the number of the action you want to take: (1-4) ");

        let mut selection = String::new();
        std::io::stdin().read_line(&mut selection).unwrap();
        selection = selection.trim().to_string();

        while selection.parse::<i32>().is_err() || selection.parse::<i32>().unwrap() < 1 || selection.parse::<i32>().unwrap() > 5 {
            println!("Please enter a valid number.");
            selection = String::new();
            std::io::stdin().read_line(&mut selection).unwrap();
        }

        let num = selection.parse::<i32>().expect("Failed to parse selection.");

        if num == 1 {
            println!("Enter the amount of RAM you want to allocate to the server in gigabytes: ");

            let mut ram_input = String::new();
            std::io::stdin().read_line(&mut ram_input).unwrap();
            ram_input = ram_input.trim().to_string();

            while ram_input.parse::<i32>().is_err() || ram_input.parse::<i32>().unwrap() < 1 {
                println!("Please enter a valid number.");
                ram_input = String::new();
                std::io::stdin().read_line(&mut ram_input).unwrap();
            }

            let ram = ram_input.parse::<i32>().expect("Failed to parse RAM.");

            if os == "windows" {
                create_launch_bat(config.get_java_install_path().expect("Failed to get Java path from config()").as_str(), ram).await;
            } else {
                create_launch_sh(config.get_java_install_path().expect("Failed to get Java path from config()").as_str(), ram).await;
            }

            println!("Launch script was created!");
            return;
        } else if num == 3   {
            goodbye();
            return;
        }
    }

    let java_key = if is_arm {
        os.to_string() + "_arm"
    } else {
        os.to_string()
    };

    let client = Client::new();

    println!();
    println!("What kind of server do you want to run?");
    println!("1. Vanilla - The original Minecraft server. No plugins or mods.");
    println!("2. Paper - A Minecraft server with plugins.");
    println!("3. FabricMC - A Minecraft server with Fabric mods.");
    println!();
    println!("Enter the number of the server you want to run: (1-3) ");

    let mut server_type = String::new();
    std::io::stdin().read_line(&mut server_type).unwrap();
    server_type = server_type.trim().to_string();

    while server_type.parse::<i32>().is_err()
        || server_type.parse::<i32>().unwrap() < 1
        || server_type.parse::<i32>().unwrap() > 4
    {
        println!("Please enter a valid number.");
        server_type = String::new();
        std::io::stdin().read_line(&mut server_type).unwrap();
    }

    let num = server_type.parse::<i32>().expect("Failed to parse server type.");

    println!();
    println!("What version of Minecraft do you want to run?");
    println!("Type latest for the latest version.");

    let mut minecraft_version = String::new();
    std::io::stdin().read_line(&mut minecraft_version).unwrap();
    minecraft_version = minecraft_version.trim().to_string();

    let version_option = if minecraft_version == "latest" {
        None
    } else {
        Some(minecraft_version.clone())
    };

    let java_version = config.get_java_version(minecraft_version.clone()).await.expect("Failed to get Java version.");
    let java_install_path = &config.get_java_install_path().expect("Failed to get Java path from config.");

    let java_path = java_install_path.to_string() + &config.get_java_path(os.to_string(), java_version).expect("Failed to get Java path from config.");

    println!("Using Java {}", java_version);

    download_java(&client, java_install_path.as_str(), java_path.as_str(), &config.get_java_download(java_key, java_version).unwrap().as_str())
        .await
        .expect("Failed to download Java.");

    if num == 1 {
        Vanilla::download(client.clone(), version_option)
            .await
            .expect("Failed to download Vanilla.");
    } else if num == 2 {
        Paper::download(client.clone(), version_option)
            .await
            .expect("Failed to download Paper.");
    } else if num == 3 {
        Fabric::download(client.clone(), version_option)
            .await
            .expect("Failed to download Fabric");
    }

    accept_eula().await;

    println!();
    println!("Do you want to create a launch script? (RECOMMENDED FOR NEW USERS) (y/n)");

    if yes_or_no() {
        if cfg!(target_os = "windows") {
            create_launch_bat(java_path.as_str(), 3).await;
        } else {
            create_launch_sh(java_path.as_str(), 3).await;
        }
    }

    println!();
    println!("Your server is ready to go!");
    println!("In order to allow other people to join, you will need to port forward your server.");
    println!("If you need help with port forwarding, Google how to with your router!");
    println!("If you need help with anything else, contact me on Discord: @loudbook");
    println!();
}

fn goodbye() {
    println!("Hava a nice day!");
    println!("Tool was created by Loudbook, contact me on Discord: @loudbook");
    println!();
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

async fn create_launch_bat(java_path: &str, ram: i32) {
    println!("Creating launch script...");

    let file = File::create("./launch.bat").unwrap();
    let mut file = std::io::BufWriter::new(file);

    file.write_all(
        format!(
            "\"{}\" -Xms1024M -Xmx{}G -jar server.jar nogui\npause",
            java_path,
            ram
        )
        .as_bytes(),
    )
    .unwrap();

    println!("Launch script was created!");
    println!("To start your server, double click on launch.bat");
}

async fn create_launch_sh(java_path: &str, ram: i32) {
    println!("Creating launch script...");

    let file = File::create("./launch.sh").unwrap();
    let mut file = std::io::BufWriter::new(file);

    file.write_all(
        format!(
            "\"{}\" -Xms1024M -Xmx{}G -jar server.jar nogui\npause",
            java_path,
            ram
        )
        .as_bytes(),
    )
    .unwrap();

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

fn extract(file: &File, path: &str) {
    if cfg!(target_os = "windows") {
        let mut archive = zip::ZipArchive::new(file).expect("Failed to create ZipArchive.");
        archive.extract(path).expect("Failed to extract Java file.");
    } else {
        let decompressed = GzDecoder::new(file);

        let mut archive = Archive::new(decompressed);
        archive.unpack(path).expect("Failed to extract Java file.");
    }
}
