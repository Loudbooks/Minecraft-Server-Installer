extern crate core;

pub mod downloaders;
pub mod config;
pub mod downloader;
pub mod downloaderror;

use flate2::read::GzDecoder;
use reqwest::Client;
use std::{env, fs, panic};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, stdin, stdout, Write};
use std::process::{Command, exit, Stdio};
use tar::Archive;
use crate::downloader::Downloader;
use crate::downloaders::fabric::Fabric;
use crate::downloaders::{forge, neoforge};
use crate::downloaders::forge::Forge;
use crate::downloaders::java::download_java;
use crate::downloaders::neoforge::NeoForge;
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
        } else if File::open("./launch.sh").is_ok() {
            ready = true;
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
                Ok(value) => !(1..=4).contains(&value),
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
                change_ram();
                continue
            } else if num == 3 {
                change_port();
                continue
            } else if num == 4 && fs::remove_file("./server.jar").is_ok() {
                println!("Server file was removed.");
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
        println!("3. Fabric - A Minecraft server with Fabric mods.");
        println!("4. Forge - A Minecraft server with Forge mods.");
        println!("5. Neoforge - A Minecraft server with Neoforge mods.");
        println!();
        print!("Enter the number of the server you want to run: (1-5): ");

        let mut server_type = user_input();

        while match server_type.parse::<i32>() {
            Ok(value) => !(1..=5).contains(&value),
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

        download_java(&client, java_install_path.as_str(), java_path.as_str(), config.get_java_download(java_key, java_version).unwrap().as_str())
            .await
            .expect("Failed to download Java");

        println!("Beginning server download...");
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
        } else if num == 4 {
            Forge::download(client.clone(), version_option.clone()).await.expect("Failed to download Forge");
            forge::build_server(java_path.clone(), version_option).await;
        } else if num == 5 {
            NeoForge::download(client.clone(), version_option.clone()).await.expect("Failed to download Neoforge");
            neoforge::build_server(java_path.clone(), version_option).await;
        }
        
        accept_eula().await;

        println!();

        if num != 5 {
            create_launch_script(Some(java_path.as_str()), java_version, os, 3);
        } else {
            create_args_file(3);
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

fn change_ram() {
    print!("Enter the amount of RAM you want to allocate to the server in gigabytes: ");

    let mut ram_input = user_input();

    while ram_input.parse::<i32>().is_err() || ram_input.parse::<i32>().unwrap() < 1 {
        print!("Please enter a valid number: ");
        ram_input = user_input();
    }

    let ram = ram_input.parse::<i32>().expect("Failed to parse RAM");

    create_args_file(ram);
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

    input == "y"
}

fn create_launch_script(java_path: Option<&str>, java_version: i32, os: &str, ram: i32) {
    println!("Creating launch script...");
    create_args_file(ram);

    let file_name = if os == "windows" {
        "./launch.bat"
    } else {
        "./launch.sh"
    };

    let args_str = if java_version == 8 {
        let args_vec = fs::read_to_string("./user_jvm_args.txt").expect("Failed to read user_jvm_args.txt");
        let args_vec = args_vec.split_whitespace().collect::<Vec<&str>>();

        let mut args = String::new();

        for arg in args_vec {
            if !arg.starts_with('#') {
                args += (" ".to_string() + arg).as_str()
            }
        }

        args
    } else {
        "@user_jvm_args.txt".to_string()
    };

    match java_path {
        None => {
            let original_content = fs::read_to_string(file_name).expect("Failed to read launch file");
            let original_java_path = original_content.lines()
                .filter(|line| !line.starts_with('#') && !line.starts_with("REM" ) && !line.starts_with('@'))
                .collect::<Vec<&str>>().first().unwrap().split_whitespace().collect::<Vec<&str>>()[0];

            fs::remove_file(file_name).expect("Failed to remove launch file");

            let file = File::create(file_name).unwrap();
            let mut file = BufWriter::new(file);

            if os == "windows" {
                file.write_all(
                    format!(
                        "@echo off\n\"{}\" {} -jar server.jar",
                        original_java_path.replace('"', ""),
                        args_str
                    )
                        .as_bytes(),
                ).expect("Failed to write to launch file");
            } else {
                file.write_all(
                    format!(
                        "#!#!/usr/bin/env sh\n\"{}\" {} -jar server.jar",
                        original_java_path.replace('"', ""),
                        args_str
                    )
                        .as_bytes(),
                ).expect("Failed to write to launch file");
            }
        }
        Some(java_path) => {
            let file = File::create(file_name).unwrap();
            let mut file = BufWriter::new(file);

            file.write_all(
                format!(
                    "\"{}\" {} -jar server.jar",
                    java_path,
                    args_str
                )
                    .as_bytes(),
            )
                .expect("Failed to write to launch file");
        }
    }

    if os != "windows" {
        Command::new("chmod")
            .arg("+x")
            .arg("./launch.sh")
            .output()
            .expect("Failed to chmod launch.sh");
    }

    println!("Launch script was created!");
}

fn create_args_file(ram: i32) {
    match File::open("./user_jvm_args.txt") {
        Ok(_) => {
            let mut file = File::open("user_jvm_args.txt").expect("Failed to open user_jvm_args.txt");
            let mut content = String::new();
            file.read_to_string(&mut content).expect("Failed to read user_jvm_args.txt");

            let new_script = if content.contains("-Xmx") {
                content.lines().collect::<Vec<&str>>().iter().map(|s| {
                    if s.contains("-Xmx") && !s.starts_with('#') {
                        format!("-Xmx{}G", ram)
                    } else {
                        s.to_string()
                    }
                }).collect::<Vec<String>>().join("\n")
            } else {
                format!("{} -Xms1024M -Xmx{}G", content, ram)
            };

            fs::write("user_jvm_args.txt", new_script).expect("Failed to write to user_jvm_args.txt");
        }
        Err(_) => {
            let file = File::create("./user_jvm_args.txt").unwrap();
            let mut file = BufWriter::new(file);

            file.write_all(
                format!(
                    "-Xms1024M -Xmx{}G",
                    ram
                )
                    .as_bytes(),
            )
                .expect("Failed to write to user_jvm_args.txt");
        }
    };
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

    let args_no_comments = content.lines().filter(
        |line| !line.starts_with('#') && !line.starts_with("REM") && !line.starts_with('@')).collect::<Vec<&str>>();

    let java_path = &args_no_comments.clone().first().unwrap().split_whitespace().collect::<Vec<&str>>()[0].replace('"', "");
    let args = &args_no_comments.first().unwrap().split_whitespace().collect::<Vec<&str>>()[1..];

    println!("Starting server with Java path: {}", java_path);

    let mut process = Command::new(java_path)
        .args(args.iter().map(|s| s.replace('"', "")).collect::<Vec<String>>())
        .stdout(Stdio::piped())
        .spawn().expect("Failed to start server");

    let out = process.stdout.take()
        .expect("Failed to capture standard output");
    let reader = BufReader::new(out);

    let mut port: Option<u32> = None;
    for line in reader.lines() {
        let line = line;

        if let Ok(line) = line {
            println!("{}", line);

            if line.contains("Starting Minecraft server on *:") {
                let parsed_port = line.split("*:").collect::<Vec<&str>>()[1].parse::<u32>().expect("Failed to parse port");
                println!("Port successfully parsed: {}", parsed_port);

                port = Some(parsed_port);
            }

            if line.contains("Done (") {
                println!();
                println!("Server is ready!");
                println!("To safely stop the server, type 'stop' and press enter.");

                if port.is_some() {
                    let ip = if port == Some(25565) {
                        format!("{}", public_ip::addr().await.unwrap())
                    } else {
                        format!("{}:{}", public_ip::addr().await.unwrap(), port.unwrap())
                    };

                    println!("If you port forwarded your server, other people can join using the following IP: {}", ip);
                }

                println!();
            }
        }
    }

    process.wait().expect("Failed to wait for server to finish");
    wait_for_enter("continue");
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
            print!("Enter the new port you want to use: ");

            let mut new_port = user_input();

            while new_port.parse::<i32>().is_err() || new_port.parse::<i32>().unwrap() < 1 || new_port.parse::<i32>().unwrap() > 65535 {
                print!("Please enter a valid port: ");
                new_port = user_input();
            }

            new_lines.push(line.split('=').collect::<Vec<&str>>()[0].to_string() + format!("={}", new_port).as_str());
        } else {
            new_lines.push(line.to_string());
        }
    }

    let file = File::create("./server.properties").expect("Failed to create server.properties");
    let mut file = BufWriter::new(file);

    for mut line in new_lines {
        line += "\n";
        file.write_all(line.as_bytes()).expect("Failed to write to server.properties");
    }

    println!("Port was changed!");
    wait_for_enter("continue");
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

    input
}
