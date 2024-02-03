extern crate core;

pub mod downloaders;
pub mod config;
pub mod downloader;
pub mod downloaderror;
pub mod os;
mod servertype;

use flate2::read::GzDecoder;
use reqwest::Client;
use std::{env, fs, panic};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Read, stdin, stdout, Write};
use std::net::SocketAddrV4;
use std::ops::{Deref};
use std::path::Path;
use std::process::{Command, exit, Stdio};
use tar::Archive;
use crate::downloader::Installer;
use crate::downloaders::fabric::Fabric;
use crate::downloaders::bungeecord::BungeeCord;
use crate::downloaders::forge::Forge;
use crate::downloaders::geyser::Geyser;
use crate::downloaders::java::download_java;
use crate::downloaders::neoforge::NeoForge;
use crate::downloaders::paper::Paper;
use crate::downloaders::vanilla::Vanilla;
use crate::downloaders::velocity::Velocity;
use crate::downloaders::waterfall::Waterfall;
use crate::os::OS;
use crate::servertype::ServerType::{Proxy, Server};

#[tokio::main]
async fn main() {
    prepare_hook();

    let downloaders: Vec<Box<dyn Installer>> = vec![
        Box::new(Vanilla {}),
        Box::new(Paper {}),
        Box::new(Fabric {}),
        Box::new(Forge {}),
        Box::new(NeoForge {}),
        Box::new(Geyser {}),
        Box::new(BungeeCord {}),
        Box::new(Velocity {}),
        Box::new(Waterfall {}),
    ];

    let is_arm = env::consts::ARCH.contains("arch64") || env::consts::ARCH.contains("arm");
    let os = if cfg!(target_os = "macos") {
        OS::MacOS
    } else if cfg!(target_os = "linux") {
        OS::Linux
    } else if cfg!(target_os = "windows") {
        OS::Windows
    } else {
        panic!("Unsupported OS.");
    };

    let config = config::ConfigFile {
        path: if os == OS::Windows {
            env::var("APPDATA").unwrap_or("./".to_string()) + "\\MinecraftServerInstaller"
        } else if os == OS::Linux {
            env::var("XDG_CONFIG_HOME").unwrap_or("./".to_string()) + "/MinecraftServerInstaller"
        } else {
            env::var("HOME").unwrap_or("./".to_string()) + "/Library/Application Support/MinecraftServerInstaller"
        }
    };

    if !Path::new(&(config.path.to_string() + "/msi-config.toml")).exists() {
        if !Path::new(&config.path).exists() {
            fs::create_dir_all(&config.path).expect("Failed to create config directory");
        }

        config.create();
    }

    config.test();

    println!("Welcome to the Minecraft Server Installer!");
    println!("This tool will help you set up a Minecraft server with ease.");
    println!();
    println!("If at any time you want to exit, type 'exit'.");
    println!();

    wait_for_enter("continue");
    loop {
        let mut ready = false;

        if os == OS::Windows {
            if File::open("./launch.bat").is_ok() {
                ready = true;
            }
        } else if File::open("./launch.sh").is_ok() {
            ready = true;
        }

        if File::open("./server.jar").is_ok() {
            ready = true;
        }

        if get_selected_from_cache(&downloaders).is_none() {
            ready = false;
        }

        if ready {
            let server_object = get_selected_from_cache(&downloaders).unwrap();

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
                run_launch_file(&os, server_object).await;
                continue
            } else if num == 2 {
                change_ram();
                continue
            } else if num == 3 {
                change_port(server_object);
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
        println!("Servers:" );
        let server_downloaders = downloaders.iter().filter(|downloader| downloader.get_type() == Server).collect::<Vec<&Box<dyn Installer>>>();
        for (mut index, downloader) in server_downloaders.iter().enumerate() {
            index += 1;
            println!("  {}. {} - {}", index, downloader.get_name(), downloader.get_description());
        }

        println!("Proxies:");
        let proxy_downloaders = downloaders.iter().filter(|downloader| downloader.get_type() == Proxy).collect::<Vec<&Box<dyn Installer>>>();
        for (mut index, downloader) in proxy_downloaders.iter().enumerate() {
            index += 1;
            println!("  {}. {} - {}", index + server_downloaders.len(), downloader.get_name(), downloader.get_description());
        }

        println!();
        print!("Enter the number of the server you want to run: (1-{}): ", downloaders.len());

        let mut server_type = user_input();
        let total_types: i32 = downloaders.len() as i32;

        while match server_type.parse::<i32>() {
            Ok(value) => !(1..=total_types).contains(&value),
            Err(_) => true,
        } {
            print!("Please enter a valid number: ");
            server_type = user_input();
        }

        let num = server_type.parse::<i32>().expect("Failed to parse server type");
        let server_object = downloaders.get((num - 1) as usize).expect("Failed to get server object");

        let minecraft_version = if server_object.version_required() {
            println!();
            print!("What version of Minecraft do you want to run? Type latest for the latest version: ");

            let input = user_input();

            if input == "latest" {
                None
            } else {
                Some(input)
            }
        } else {
            None
        };

        println!("Beginning download...");

        let java_version = config.get_java_version(minecraft_version.clone()).await.expect("Failed to get Java version");
        let java_install_path = &config.get_java_install_path().expect("Failed to get Java path from config");

        let java_path = java_install_path.to_string() + &config.get_java_path(os.to_string(), java_version).expect("Failed to get Java path from config");

        println!("Using Java {}", java_version);

        download_java(&client, java_install_path.as_str(), java_path.as_str(), config.get_java_download(java_key, java_version).unwrap().as_str(), &os)
            .await
            .expect("Failed to download Java");

        println!("Beginning server download...");

        server_object.download(client.clone(), minecraft_version.clone()).await.expect("Failed to download server");
        server_object.build(java_path.clone(), minecraft_version.clone()).await;
        
        accept_eula().await;

        if server_object.custom_script() {
            create_args_file(3);
        } else {
            create_launch_script(Some(java_path.as_str()), java_version, &os, 3);
        }

        save_selected_cache(server_object.deref());

        println!();
        println!("Your server is ready to go!");
        println!("In order to allow other people to join, you will need to port forward your server.");
        println!("If you need help with port forwarding, Google how to with your router!");
        println!("If you need help with anything else, contact me on Discord: @loudbook");
        println!();

        print!("Would you like to run your server now? (y/n): ");

        if yes_or_no() {
            run_launch_file(&os, server_object.deref()).await;
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

fn create_launch_script(java_path: Option<&str>, java_version: i32, os: &OS, ram: i32) {
    println!("Creating launch script...");
    create_args_file(ram);

    let file_name = if os == &OS::Windows {
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

            if os == &OS::Windows {
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

    if os != &OS::Windows {
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

fn extract(file: &File, path: &str, os: &OS) {
    if os == &OS::Windows {
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

async fn run_launch_file(os: &OS, server: &dyn Installer) {
    println!("Starting server...");

    let mut content = String::new();

    if os == &OS::Windows {
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

    let mut address: Option<SocketAddrV4> = None;
    for line in reader.lines() {
        let line = line;

        if let Ok(line) = line {
            println!("{}", line);

            if server.startup_message(line.clone()).await.is_some() {
                address = server.startup_message(line.clone()).await;
            }

            if line.contains("Done (") || line.contains("Listening on /") {
                println!();
                println!("Server is ready!");
                println!("To safely stop the server, type 'stop' and press enter.");

                if address.is_some() {
                    println!("If you port forwarded your server, other people can join using the following IP: {}", address.unwrap());
                }

                println!();
            }
        }
    }

    process.wait().expect("Failed to wait for server to finish");
    wait_for_enter("continue");
}

fn change_port(server: &dyn Installer) {
    if server.get_name() == "Geyser" {
        let config = fs::read_to_string("./config.yml").expect("config.yml not found. Make sure you have run the server at least once!");
        let config = config.split('\n').collect::<Vec<&str>>();

        let mut new_config: Vec<String> = Vec::new();

        let mut section = "Bedrock";

        for line in config {
            if line.starts_with("  port:") {
                print!("Enter the new port you want to use for {}: ", section);

                let mut new_port = user_input();

                while new_port.parse::<i32>().is_err() || new_port.parse::<i32>().unwrap() < 1 || new_port.parse::<i32>().unwrap() > 65535 {
                    print!("Please enter a valid port: ");
                    new_port = user_input();
                }

                let host_string = format!("  port: {}", new_port);
                let final_string = host_string;

                new_config.push(final_string);

                section = "Java";
            } else {
                new_config.push(line.to_string());
            }
        }

        let file = File::create("./config.yml").expect("Failed to create config.yml");
        let mut file = BufWriter::new(file);

        for mut line in new_config {
            line += "\n";
            file.write_all(line.as_bytes()).expect("Failed to write to config.yml");
        }

        return
    } else if server.get_name() == "BungeeCord" || server.get_name() == "Waterfall" {
        let config = fs::read_to_string("./config.yml").expect("config.yml not found. Make sure you have run the server at least once!");
        let config = config.split('\n').collect::<Vec<&str>>();

        let mut new_config: Vec<String> = Vec::new();

        print!("Enter the new port you want to use: ");

        let mut new_port = user_input();

        while new_port.parse::<i32>().is_err() || new_port.parse::<i32>().unwrap() < 1 || new_port.parse::<i32>().unwrap() > 65535 {
            print!("Please enter a valid port: ");
            new_port = user_input();
        }

        for line in config {
            if line.starts_with("  host:") {
                let host_string = format!("  host: 0.0.0.0:{}", new_port);
                let final_string = host_string;

                new_config.push(final_string);
            } else {
                new_config.push(line.to_string());
            }
        }

        let file = File::create("./config.yml").expect("Failed to create config.yml");
        let mut file = BufWriter::new(file);

        for mut line in new_config {
            line += "\n";
            file.write_all(line.as_bytes()).expect("Failed to write to config.yml");
        }

        return
    } else if server.get_name() == "Velocity" {
        let toml = fs::read_to_string("./velocity.toml").expect("velocity.toml not found. Make sure you have run the server at least once!");
        let toml = toml.split('\n').collect::<Vec<&str>>();

        let mut new_toml: Vec<String> = Vec::new();

        print!("Enter the new port you want to use: ");

        let mut new_port = user_input();

        while new_port.parse::<i32>().is_err() || new_port.parse::<i32>().unwrap() < 1 || new_port.parse::<i32>().unwrap() > 65535 {
            print!("Please enter a valid port: ");
            new_port = user_input();
        }

        for line in toml {
            if line.starts_with("bind") {
                let bind_string =  format!("bind = \"0.0.0.0:{}\"", new_port);
                let final_string = bind_string;

                new_toml.push(final_string);
            } else {
                new_toml.push(line.to_string());
            }
        }

        let file = File::create("./velocity.toml").expect("Failed to create velocity.toml");
        let mut file = BufWriter::new(file);

        for mut line in new_toml {
            line += "\n";
            file.write_all(line.as_bytes()).expect("Failed to write to velocity.toml");
        }

        return
    }

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

fn save_selected_cache(server: &dyn Installer) {
    let file = File::create("./selected_cache.txt").expect("Failed to create selected_cache.txt");
    let mut file = BufWriter::new(file);

    file.write_all(server.get_name().as_bytes()).expect("Failed to write to selected_cache.txt");
}

fn get_selected_from_cache(options: &Vec<Box<dyn Installer>>) -> Option<&dyn Installer> {
    let file = File::open("./selected_cache.txt");

    if file.is_err() {
        return None
    }

    let mut file = BufReader::new(file.unwrap());
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Failed to read selected_cache.txt");

    for installer in options {
        if installer.get_name() == content {
            return Some(installer.deref())
        }
    }

    None
}
