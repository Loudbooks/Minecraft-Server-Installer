extern crate core;

pub mod downloaders;
pub mod config;
pub mod downloader;
pub mod downloaderror;

use flate2::read::GzDecoder;
use reqwest::Client;
use std::{env, fs, panic};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, stdin, stdout, Write};
use std::process::{Command, exit};
use tar::Archive;
use crate::downloader::Downloader;
use crate::downloaders::fabric::Fabric;
use crate::downloaders::java::download_java;
use crate::downloaders::paper::Paper;
use crate::downloaders::vanilla::Vanilla;

#[tokio::main]
async fn main() {
    prepare_hook();

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
            env::var("APPDATA").expect("Failed to retrieve APPDATA variable") + "\\MinecraftServerInstaller"
        } else if os == "linux" {
            env::var("XDG_CONFIG_HOME").expect("Failed to retrieve XDG_CONFIG_HOME variable") + "/MinecraftServerInstaller"
        } else {
            env::var("HOME").expect("Failed to retrieve HOME variable") + "/Library/Application Support/MinecraftServerInstaller"
        }
    };

    if !std::path::Path::new(&(config.path.to_string() + "/msi-config.toml")).exists() {
        if !std::path::Path::new(&config.path).exists() {
            fs::create_dir(&config.path).expect("Failed to create config directory");
        }

        config.create();
    }

    println!("Welcome to the Minecraft Server Installer!");
    println!("This tool will help you set up a Minecraft server with ease.");
    println!();
    println!("If at any time you want to exit, type 'exit'.");
    println!();

    wait_for_enter("continue");
    loop {
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
            println!("A valid server file was found.");
            println!("1. Run the server.");
            println!("2. Change amount of RAM allocated to the server.");
            println!("3. Change the running port of the server.");
            println!("4. Replace your server file.");
            println!();
            print!("Enter the number of the action you want to take: (1-4): ");

            let mut selection = user_input();

            while match selection.parse::<i32>() {
                Ok(value) => value < 1 || value > 4,
                Err(_) => true,
            } {
                print!("Please enter a valid number: ");
                selection = user_input();
            }

            let num = selection.parse::<i32>().expect("Failed to parse selection");

            if num == 1 {
                run_launch_file(os).await;
                continue
            } else if num == 2 {
                change_ram(os);
                continue
            } else if num == 3 {
                change_port();
                continue
            } else if num == 4 {
                if let Ok(_) = fs::remove_file("./server.jar") {
                    println!("Server file was removed.");
                }
            }
        }

        let java_key = if is_arm {
            os.to_string() + "_arm"
        } else {
            os.to_string()
        };

        let client = Client::new();

        println!("What kind of server do you want to run?");
        println!("1. Vanilla - The original Minecraft server. No plugins or mods.");
        println!("2. Paper - A Minecraft server with plugins.");
        println!("3. FabricMC - A Minecraft server with Fabric mods.");
        println!();
        print!("Enter the number of the server you want to run: (1-3): ");

        let mut server_type = user_input();

        while match server_type.parse::<i32>() {
            Ok(value) => value < 1 || value > 3,
            Err(_) => true,
        } {
            print!("Please enter a valid number: ");
            server_type = user_input();
        }

        let num = server_type.parse::<i32>().expect("Failed to parse server type");

        println!();
        print!("What version of Minecraft do you want to run? Type latest for the latest version: ");

        let minecraft_version = user_input();

        let version_option = if minecraft_version == "latest" {
            None
        } else {
            Some(minecraft_version.clone())
        };

        println!("Beginning download...");

        let java_version = config.get_java_version(minecraft_version.clone()).await.expect("Failed to get Java version");
        let java_install_path = &config.get_java_install_path().expect("Failed to get Java path from config");

        let java_path = java_install_path.to_string() + &config.get_java_path(os.to_string(), java_version).expect("Failed to get Java path from config");

        println!("Using Java {}", java_version);

        download_java(&client, java_install_path.as_str(), java_path.as_str(), &config.get_java_download(java_key, java_version).unwrap().as_str())
            .await
            .expect("Failed to download Java");

        if num == 1 {
            Vanilla::download(client.clone(), version_option)
                .await
                .expect("Failed to download Vanilla");
        } else if num == 2 {
            Paper::download(client.clone(), version_option)
                .await
                .expect("Failed to download Paper");
        } else if num == 3 {
            Fabric::download(client.clone(), version_option)
                .await
                .expect("Failed to download Fabric");
        }

        accept_eula().await;

        println!();
        print!("Do you want to create a launch script? (RECOMMENDED FOR NEW USERS) (y/n): ");

        if yes_or_no() {
            create_launch_script(Some(java_path.as_str()), os, 3);
        }

        println!();
        println!("Your server is ready to go!");
        println!("In order to allow other people to join, you will need to port forward your server.");
        println!("If you need help with port forwarding, Google how to with your router!");
        println!("If you need help with anything else, contact me on Discord: @loudbook");
        println!();

        print!("Would you like to run your server now? (y/n): ");

        if yes_or_no() {
            run_launch_file(os).await;
        } else {
            goodbye();
            wait_for_enter("exit");
            exit(0)
        }
    }
}

fn change_ram(os: &str) {
    print!("Enter the amount of RAM you want to allocate to the server in gigabytes: ");

    let mut ram_input = user_input();

    while ram_input.parse::<i32>().is_err() || ram_input.parse::<i32>().unwrap() < 1 {
        println!("Please enter a valid number.");
        ram_input = user_input();
    }

    let ram = ram_input.parse::<i32>().expect("Failed to parse RAM");

    create_launch_script(None, os, ram);

    println!("Launch script was created!");
}

fn goodbye() {
    println!("Hava a nice day!");
    println!("Tool was created by Loudbook, contact me on Discord: @loudbook");
    println!();
}

fn yes_or_no() -> bool {
    let mut input = user_input();

    while input != "y" && input != "n" {
        print!("Please enter y or n: ");
        input = user_input();
    }

    return input == "y";
}

fn create_launch_script(java_path: Option<&str>, os: &str, ram: i32) {
    println!("Creating launch script...");

    let file_name = if os == "windows" {
        "./launch.bat"
    } else {
        "./launch.sh"
    };

    match java_path {
        None => {
            let original_content = fs::read_to_string(file_name).expect("Failed to read launch file");
            let original_java_path = original_content.split_whitespace().collect::<Vec<&str>>()[0];

            fs::remove_file(file_name).expect("Failed to remove launch file");

            let file = File::create(file_name).unwrap();
            let mut file = BufWriter::new(file);

            file.write_all(
                format!(
                    "{} -Xms1024M -Xmx{}G -jar server.jar nogui\npause",
                    original_java_path,
                    ram
                )
                    .as_bytes(),
            ).expect("Failed to write to launch file");
        }
        Some(java_path) => {
            let file = File::create(file_name).unwrap();
            let mut file = BufWriter::new(file);

            file.write_all(
                format!(
                    "\"{}\" -Xms1024M -Xmx{}G -jar server.jar nogui\npause",
                    java_path,
                    ram
                )
                    .as_bytes(),
            )
                .expect("Failed to write to launch file")
        }
    }

    if os == "windows" {
        println!("Launch script was created!");
    } else {
        Command::new("chmod")
            .arg("+x")
            .arg("./launch.sh")
            .output()
            .expect("Failed to chmod launch.sh");

        println!("Launch script was created!");
    }
}

async fn accept_eula() {
    println!("Checking EULA...");
    let file = File::create("./eula.txt").unwrap();
    let mut file = BufWriter::new(file);

    file.write_all("eula=true".as_bytes()).unwrap();
}

fn extract(file: &File, path: &str) {
    if cfg!(target_os = "windows") {
        let mut archive = zip::ZipArchive::new(file).expect("Failed to create ZipArchive");
        archive.extract(path).expect("Failed to extract Java file");
    } else {
        let decompressed = GzDecoder::new(file);

        let mut archive = Archive::new(decompressed);
        archive.unpack(path).expect("Failed to extract Java file");
    }
}

fn prepare_hook() {
    panic::set_hook(Box::new(|panic_info| {
        println!();
        println!("Error: {}", panic_info);
        println!();
        println!("Please report this error to me on Discord: @loudbook");
        println!();

        wait_for_enter("exit");
    }));
}

fn wait_for_enter(message: &str) {
    println!("Press enter to {}.", message);
    let _ = stdin().read_line(&mut String::new());
}

async fn run_launch_file(os: &str) {
    println!("Starting server...");

    let mut content = String::new();

    if os == "windows" {
        File::open("./launch.bat").expect("Failed to open launch.bat").read_to_string(&mut content).expect("Failed to read launch.bat");
    } else {
        File::open("./launch.sh").expect("Failed to open launch.sh").read_to_string(&mut content).expect("Failed to read launch.sh");
    };

    let java_path = content.split_whitespace().collect::<Vec<&str>>()[0].replace("\"", "");
    let args = &content.split_whitespace().collect::<Vec<&str>>()[1..];

    Command::new(java_path).args(args.iter()).spawn().expect("Failed to start server").wait().expect("Failed to wait for server to start");
}

fn change_port() {
    let file = File::open("./server.properties").expect("Failed to open server.properties");
    let mut file = BufReader::new(file);

    let mut content = String::new();
    file.read_to_string(&mut content).expect("Failed to read server.properties");

    let lines = content.lines().collect::<Vec<&str>>();
    let mut new_lines = Vec::new();

    for line in lines {
        if line.contains("server-port=") {
            println!("Enter the new port you want to use: ");

            let mut new_port = user_input();

            while new_port.parse::<i32>().is_err() || new_port.parse::<i32>().unwrap() < 49152 || new_port.parse::<i32>().unwrap() > 65535 {
                println!("Please enter a valid port number. (49152-65535)");
                new_port = user_input();
            }

            new_lines.push(line.split("=").collect::<Vec<&str>>()[0].to_string() + format!("={}", new_port).as_str());
        } else {
            new_lines.push(line.to_string());
        }
    }

    let file = File::create("./server.properties").expect("Failed to create server.properties");
    let mut file = BufWriter::new(file);

    for mut line in new_lines {
        line = line + "\n";
        file.write_all(line.as_bytes()).expect("Failed to write to server.properties");
    }

    println!("Port was changed!");
}

fn user_input() -> String {
    let mut input= String::new();

    stdout().flush().expect("Failed to flush");
    stdin().read_line(&mut input).expect("Did not enter a correct string");

    input = input.trim().to_string();

    if input == "exit" {
        goodbye();
        exit(0);
    }

    return input;
}
