use async_trait::async_trait;
use crate::downloader::{basic_proxy_address_from_string, download_file, Installer};
use crate::servertype::ServerType;
use crate::servertype::ServerType::Proxy;

pub(crate) struct BungeeCord {}

#[async_trait]
impl Installer for BungeeCord {
    fn get_name(&self) -> String {
        "BungeeCord".to_string()
    }

    fn get_description(&self) -> String {
        "A server that supports BungeeCord plugins.".to_string()
    }

    fn get_type(&self) -> ServerType {
        Proxy
    }

    fn custom_script(&self) -> bool {
        false
    }

    async fn startup_message(&self, string: String) -> Option<std::net::SocketAddrV4> {
        basic_proxy_address_from_string(string).await
    }

    async fn download(&self, client: reqwest::Client, _minecraft_version: Option<String>) -> Result<String, crate::downloaderror::DownloadError> {
        download_file(&client, "https://ci.md-5.net/job/BungeeCord/lastSuccessfulBuild/artifact/bootstrap/target/BungeeCord.jar", "./server.jar").await?;

        Ok("".to_string())
    }
}