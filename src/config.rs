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
    osx_8: String,
    osx_16: String,
    osx_17: String,
    linux_8: String,
    linux_16: String,
    linux_17: String,
    windows_8: String,
    windows_16: String,
    windows_17: String,
}

#[derive(Deserialize, Serialize)]
struct JavaDownloads {
    osx_8: String,
    osx_16: String,
    osx_17: String,
    osx_arm_8: String,
    osx_arm_16: String,
    osx_arm_17: String,
    linux_8: String,
    linux_16: String,
    linux_17: String,
    linux_arm_8: String,
    linux_arm_16: String,
    linux_arm_17: String,
    windows_8: String,
    windows_16: String,
    windows_17: String,
}

#[derive(Deserialize, Serialize)]
pub struct JavaVersionThresholds {
    java_16: String,
    java_17: String,
}

impl ConfigFile {
    pub fn create(&self) {
        let path = self.path.clone().to_string();
        let file = File::create(format!("{path}/msi-config.toml")).expect("Failed to create config file");

        let mut file = std::io::BufWriter::new(file);

        let toml_config = toml::to_string(&self.default_config()).expect("Failed to convert config to TOML");
        file.write_all(toml_config.as_bytes()).expect("Failed to write config to file");
    }

    pub fn get_java_download(&self, key: String, version: i32) -> Option<String> {
        let config = self.clone().get_config();

        let java_downloads = config.get("java_downloads").expect("Failed to get java_downloads");
        let java_download = java_downloads.get(key + "_" + version.to_string().as_str()).expect("Failed to get java_download key").as_str().expect("Failed to get java_download as string").to_string();

        return Some(java_download);
    }

    pub fn get_java_path(&self, key: String, version: i32) -> Option<String> {
        let config = self.get_config();

        let java_paths = config.get("java_paths").expect("Failed to get java_paths");

        return Some(java_paths.get(key + "_" + version.to_string().as_str()).expect("Failed to get java_path key").as_str().expect("Failed to get java_path as string").to_string());
    }

    pub fn get_java_install_path(&self) -> Option<String> {
        let config = self.get_config();

        let java_install_paths = config.get("java_paths").expect("Failed to get java_install_paths");

        return Some(java_install_paths.get("java_install_paths").expect("Failed to get java_install_path key").as_str().expect("Failed to get java_install_path as string").to_string());
    }

    pub fn get_java_version_threshold(&self, key: String) -> Option<String> {
        let config = self.get_config();

        let java_version_thresholds = config.get("java_version_thresholds").expect("Failed to get java_version_thresholds");

        return Some(java_version_thresholds.get(key).expect("Failed to get java_version key").as_str().expect("Failed to get java_version as string").to_string());
    }

    pub async fn get_java_version(&self, minecraft_version: String) -> Option<i32> {
        let version_index = downloader::version_index(minecraft_version).await.expect("Failed to get version index");
        let java_17_index = downloader::version_index(self.get_java_version_threshold("java_17".to_string())
            .or(Some(self.default_config().java_version_thresholds.java_17.to_string())).expect("Failed to get default version for Java 17"))
            .await.expect("Failed to get version index for Java 17");
        let java_16_index = downloader::version_index(self.get_java_version_threshold("java_16".to_string())
            .or(Some(self.default_config().java_version_thresholds.java_16.to_string())).expect("Failed to get default version for Java 16"))
            .await.expect("Failed to get version index for Java 16");

        return if version_index >= java_17_index {
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

        return config;
    }

    fn default_config(&self) -> Config {
        return Config {
            java_paths: JavaPaths {
                java_install_paths: "./java".to_string(),
                osx_8: "/jdk8u402-b06-jre/Contents/Home/bin/java".to_string(),
                osx_16: "/jdk-16.0.2+7-jre/Contents/Home/bin/java".to_string(),
                osx_17: "/jdk-17.0.10+7-jre/Contents/Home/bin/java".to_string(),
                linux_8: "/jdk8u402-b06-jre/bin/java".to_string(),
                linux_16: "/jdk-16.0.2+7-jre/bin/java".to_string(),
                linux_17: "/jdk-17.0.10+7-jre/bin/java".to_string(),
                windows_8: "/jdk8u402-b06-jre/bin/java.exe".to_string(),
                windows_16: "/jdk-16.0.2+7-jre/bin/java.exe".to_string(),
                windows_17: "/jdk-17.0.10+7-jre/bin/java.exe".to_string(),
            },
            java_downloads: JavaDownloads {
                osx_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_x64_mac_hotspot_8u402b06.tar.gz".to_string(),
                osx_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jre_x64_mac_hotspot_2021-09-14-01-32.tar.gz".to_string(),
                osx_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_mac_hotspot_17.0.10_7.tar.gz".to_string(),
                osx_arm_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_x64_mac_hotspot_8u402b06.tar.gz".to_string(),
                osx_arm_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jre_aarch64_linux_hotspot_2021-09-14-01-32.tar.gz".to_string(),
                osx_arm_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_aarch64_mac_hotspot_17.0.10_7.tar.gz".to_string(),
                linux_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_x64_linux_hotspot_8u402b06.tar.gz".to_string(),
                linux_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jre_x64_linux_hotspot_2021-09-14-01-32.tar.gz".to_string(),
                linux_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_linux_hotspot_17.0.10_7.tar.gz".to_string(),
                linux_arm_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_aarch64_linux_hotspot_8u402b06.tar.gz".to_string(),
                linux_arm_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jdk_aarch64_linux_hotspot_2021-09-14-01-32.tar.gz".to_string(),
                linux_arm_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_arm_linux_hotspot_17.0.10_7.tar.gz".to_string(),
                windows_8: "https://github.com/adoptium/temurin8-binaries/releases/download/jdk8u402-b06/OpenJDK8U-jre_x64_windows_hotspot_8u402b06.zip".to_string(),
                windows_16: "https://github.com/adoptium/temurin16-binaries/releases/download/jdk16u-2021-09-14-01-32-beta/OpenJDK16U-jre_x64_windows_hotspot_2021-09-14-01-32.zip".to_string(),
                windows_17: "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.10%2B7/OpenJDK17U-jre_x64_windows_hotspot_17.0.10_7.zip".to_string(),
            },
            java_version_thresholds: JavaVersionThresholds {
                java_16: "21w19a".to_string(),
                java_17: "1.18-pre2".to_string(),
            },
        };
    }
}