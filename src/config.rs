use std::fs;
use std::fs::File;
use std::io::Write;
use serde::{Deserialize, Serialize};
use toml::Value;

#[derive(Clone)]
pub struct ConfigFile {
    pub(crate) path: String
}

#[derive(Deserialize, Serialize)]
struct Config {
    java_paths: JavaPaths,
    java_downloads: JavaDownloads
}

#[derive(Deserialize, Serialize)]
struct JavaPaths {
    osx: String,
    linux: String,
    windows: String,
}

#[derive(Deserialize, Serialize)]
struct JavaDownloads {
    osx: String,
    osx_arm: String,
    linux: String,
    linux_arm: String,
    windows: String,
}

impl ConfigFile {
    pub fn create(&self) {
        let path = self.path.clone().to_string();
        let file = File::create(format!("{path}/msi-config.toml")).expect("Failed to create config file.");

        let mut file = std::io::BufWriter::new(file);

        let default_config = Config {
            java_paths: JavaPaths {
                osx: "./java/jdk-17.0.10+7-jre/Contents/home/bin/java".to_string(),
                linux: "./java/jdk-17.0.10+7-jre/bin/java".to_string(),
                windows: "./java/jdk-17.0.10+7-jre/bin/java.exe".to_string(),
            },
            java_downloads: JavaDownloads {
                osx: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_mac_hotspot_17.0.10_7.tar.gz".to_string(),
                osx_arm: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_aarch64_mac_hotspot_17.0.10_7.tar.gz".to_string(),
                linux: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_linux_hotspot_17.0.10_7.tar.gz".to_string(),
                linux_arm: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_aarch64_linux_hotspot_17.0.10_7.tar.gz".to_string(),
                windows: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_windows_hotspot_17.0.10_7.zip".to_string(),
            }
        };

        let toml_config = toml::to_string(&default_config).expect("Failed to convert config to TOML.");
        file.write_all(toml_config.as_bytes()).expect("Failed to write config to file.");
    }

    pub fn get_java_download(self, key: String) -> Option<String> {
        let config = self.get_config();

        let java_downloads = config.get("java_downloads").expect("Failed to get java_downloads.");
        let java_download = java_downloads.get(key).expect("Failed to get java_download key.").as_str().expect("Failed to get java_download as string.").to_string();

        return Some(java_download);
    }

    pub fn get_java_path(self, key: String) -> Option<String> {
        let config = self.get_config();

        let java_paths = config.get("java_paths").expect("Failed to get java_paths.");

        return Some(java_paths.get(key).expect("Failed to get java_path key.").as_str().expect("Failed to get java_path as string.").to_string());
    }

    fn get_config(self) -> Value {
        let string = fs::read_to_string(self.path + "/msi-config.toml").expect("Failed to read config file.");
        let config: Value = toml::from_str(&string).expect("Failed to parse config file.");

        return config;
    }
}