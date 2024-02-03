
use async_trait::async_trait;
use crate::downloader::{basic_server_address_from_string, download_file, Installer};
use crate::servertype::ServerType;
use crate::servertype::ServerType::Server;

pub(crate) struct Geyser {}

#[async_trait]
impl Installer for Geyser {
    fn get_name(&self) -> String {
        "Geyser".to_string()
    }

    fn get_description(&self) -> String {
        "A server that support Bedrock <-> Java crossplay..".to_string()
    }

    fn get_type(&self) -> ServerType {
        Server
    }

    fn custom_script(&self) -> bool {
        false
    }

    fn version_required(&self) -> bool {
        false
    }

    async fn startup_message(&self, string: String) -> Option<std::net::SocketAddrV4> {
        basic_server_address_from_string(string).await
    }

    async fn download(&self, client: reqwest::Client, _minecraft_version: Option<String>) -> Result<String, crate::downloaderror::DownloadError> {
        download_file(&client, "https://download.geysermc.org/v2/projects/geyser/versions/latest/builds/latest/downloads/standalone", "./server.jar").await?;

        Ok("".to_string())
    }
}