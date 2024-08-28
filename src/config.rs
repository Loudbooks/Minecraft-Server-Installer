use std::fs;
use std::fs::File;
use std::io::Write;
use serde::{Deserialize, Serialize};
use toml::Value;
use crate::downloader;

#[derive(Clone)]
pub struct ConfigFile {
    pub(crate) path: String
}

#[derive(Deserialize, Serialize)]
struct Config {
    java_paths: JavaPaths,
    java_downloads: JavaDownloads,
    java_version_thresholds: JavaVersionThresholds,
}

#[derive(Deserialize, Serialize)]
struct JavaPaths {
    java_install_paths: String,
    macos_8: String,
    macos_16: String,
    macos_17: String,
    macos_21: String,
    linux_8: String,
    linux_16: String,
    linux_17: String,
    linux_21: String,
    windows_8: String,
    windows_16: String,
    windows_17: String,
    windows_21: String,
}

#[derive(Deserialize, Serialize)]
struct JavaDownloads {
    macos_8: String,
    macos_16: String,
    macos_17: String,
    macos_21: String,
    macos_arm_8: String,
    macos_arm_16: String,
    macos_arm_17: String,
    macos_arm_21: String,
    linux_8: String,
    linux_16: String,
    linux_17: String,
    linux_21: String,
    linux_arm_8: String,
    linux_arm_16: String,
    linux_arm_17: String,
    linux_arm_21: String,
    windows_8: String,
    windows_16: String,
    windows_17: String,
    windows_21: String,
}

#[derive(Deserialize, Serialize)]
pub struct JavaVersionThresholds {
    java_16: String,
    java_17: String,
    java_21: String,
}

impl ConfigFile {
    pub fn create(&self) {
        let path = self.path.clone().to_string();
        let file = File::create(format!("{path}/msi-config.toml")).expect("Failed to create config file");

        let mut file = std::io::BufWriter::new(file);

        let toml_config = toml::to_string(&self.default_config()).expect("Failed to convert config to TOML");
        file.write_all(toml_config.as_bytes()).expect("Failed to write config to file");
    }

    pub fn test(&self) {
        let string = fs::read_to_string(self.path.to_string() + "/msi-config.toml");

        if string.is_err() {
            self.create();
        }

        if toml::from_str::<Config>(&string.unwrap()).is_err() {
            println!("Failed to parse config file, regenerating file.");
            fs::copy(self.path.to_string() + "/msi-config.toml", self.path.to_string() + "/msi-config-old.toml").expect("Failed to backup config file");
            println!("Backed up old config file to msi-config-old.toml");

            self.create();
        }
    }

    pub fn get_java_download(&self, key: String, version: i32) -> Option<String> {
        let config = self.clone().get_config();

        let java_downloads = config.get("java_downloads").expect("Failed to get java_downloads");
        let java_download = java_downloads.get(key + "_" + version.to_string().as_str()).expect("Failed to get java_download key").as_str().expect("Failed to get java_download as string").to_string();

        Some(java_download)
    }

    pub fn get_java_path(&self, key: String, version: i32) -> Option<String> {
        let config = self.get_config();

        let java_paths = config.get("java_paths").expect("Failed to get java_paths");

        Some(java_paths.get(key + "_" + version.to_string().as_str()).expect("Failed to get java_path key").as_str().expect("Failed to get java_path as string").to_string())
    }

    pub fn get_java_install_path(&self) -> Option<String> {
        let config = self.get_config();

        let java_install_paths = config.get("java_paths").expect("Failed to get java_install_paths");

        Some(java_install_paths.get("java_install_paths").expect("Failed to get java_install_path key").as_str().expect("Failed to get java_install_path as string").to_string())
    }

    pub fn get_java_version_threshold(&self, key: String) -> Option<String> {
        let config = self.get_config();

        let java_version_thresholds = config.get("java_version_thresholds").expect("Failed to get java_version_thresholds");

        Some(java_version_thresholds.get(key).expect("Failed to get java_version key").as_str().expect("Failed to get java_version as string").to_string())
    }

    pub async fn get_java_version(&self, minecraft_version: Option<String>) -> Option<i32> {
        let version_index = downloader::version_index(minecraft_version).await.expect("Failed to get version index");
        let java_21_index = downloader::version_index(Some(self.get_java_version_threshold("java_21".to_string()))
            .or(Some(Some(self.default_config().java_version_thresholds.java_21.to_string()))).expect("Failed to get default version for Java 21"))
            .await.expect("Failed to get version index for Java 21");
        let java_17_index = downloader::version_index(Some(self.get_java_version_threshold("java_17".to_string()))
            .or(Some(Some(self.default_config().java_version_thresholds.java_17.to_string()))).expect("Failed to get default version for Java 17"))
            .await.expect("Failed to get version index for Java 17");
        let java_16_index = downloader::version_index(Some(self.get_java_version_threshold("java_16".to_string()))
            .or(Some(Some(self.default_config().java_version_thresholds.java_16.to_string()))).expect("Failed to get default version for Java 16"))
            .await.expect("Failed to get version index for Java 16");

        if version_index >= java_21_index {
            Some(21)
        } else if version_index >= java_17_index {
            Some(17)
        } else if version_index >= java_16_index {
            Some(16)
        } else {
            Some(8)
        }
    }

    fn get_config(&self) -> Value {
        let string = fs::read_to_string(self.path.to_string() + "/msi-config.toml").expect("Failed to read config file");
        let config: Value = toml::from_str(&string).expect("Failed to parse config file");

        config
    }

    fn default_config(&self) -> Config {
        Config {
            java_paths: JavaPaths {
                java_install_paths: "./java".to_string(),
                macos_8: "/jdk8u402-b06-jre/Contents/Home/bin/java".to_string(),
                macos_16: "/jdk-16.0.2+7-jre/Contents/Home/bin/java".to_string(),
                macos_17: "/jdk-17.0.10+7-jre/Contents/Home/bin/java".to_string(),
                macos_21: "/jdk-21.0.4+7-jre/Contents/Home/bin/java".to_string(),
                linux_8: "/jdk8u402-b06-jre/bin/java".to_string(),
                linux_16: "/jdk-16.0.2+7-jre/bin/java".to_string(),
                linux_17: "/jdk-17.0.10+7-jre/bin/java".to_string(),
                linux_21: "/jdk-21.0.4+7-jre/bin/java".to_string(),
                windows_8: "/jdk8u402-b06-jre/bin/java.exe".to_string(),
                windows_16: "/jdk-16.0.2+7-jre/bin/java.exe".to_string(),
                windows_17: "/jdk-17.0.10+7-jre/bin/java.exe".to_string(),
                windows_21: "/jdk-21.0.4+7-jre/bin/java.exe".to_string(),
            },
            java_downloads: JavaDownloads {
                macos_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_x64_mac_hotspot_8u402b06.tar.gz".to_string(),
                macos_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jre_x64_mac_hotspot_2021-09-14-01-32.tar.gz".to_string(),
                macos_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_mac_hotspot_17.0.10_7.tar.gz".to_string(),
                macos_21: "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.4%2B7/OpenJDK21U-jre_x64_mac_hotspot_21.0.4_7.tar.gz".to_string(),
                macos_arm_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_x64_mac_hotspot_8u402b06.tar.gz".to_string(),
                macos_arm_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jre_aarch64_linux_hotspot_2021-09-14-01-32.tar.gz".to_string(),
                macos_arm_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_aarch64_mac_hotspot_17.0.10_7.tar.gz".to_string(),
                macos_arm_21: "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.4%2B7/OpenJDK21U-jre_aarch64_mac_hotspot_21.0.4_7.tar.gz".to_string(),
                linux_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_x64_linux_hotspot_8u402b06.tar.gz".to_string(),
                linux_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jre_x64_linux_hotspot_2021-09-14-01-32.tar.gz".to_string(),
                linux_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_linux_hotspot_17.0.10_7.tar.gz".to_string(),
                linux_21: "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.4%2B7/OpenJDK21U-jdk_x64_linux_hotspot_21.0.4_7.tar.gz".to_string(),
                linux_arm_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_aarch64_linux_hotspot_8u402b06.tar.gz".to_string(),
                linux_arm_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jdk_aarch64_linux_hotspot_2021-09-14-01-32.tar.gz".to_string(),
                linux_arm_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_arm_linux_hotspot_17.0.10_7.tar.gz".to_string(),
                linux_arm_21: "https://github.com/adoptium/temurin21-binaries/releases/download/jdk-21.0.4%2B7/OpenJDK21U-jre_aarch64_linux_hotspot_21.0.4_7.tar.gz".to_string(),
                windows_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_x64_windows_hotspot_8u402b06.zip".to_string(),
                windows_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jre_x64_windows_hotspot_2021-09-14-01-32.zip".to_string(),
                windows_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_windows_hotspot_17.0.10_7.zip".to_string(),

                windows_21: "".to_string(),
            },
            java_version_thresholds: JavaVersionThresholds {
                java_16: "21w19a".to_string(),
                java_17: "1.18-pre2".to_string(),
                java_21: "24w14a".to_string(),
            },
        }
    }
}