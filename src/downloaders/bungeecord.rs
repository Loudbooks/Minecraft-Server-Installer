use crate::downloader::{basic_proxy_address_from_string, download_file, Downloader};
use crate::servertype::ServerType;
use crate::servertype::ServerType::Proxy;

pub(crate) struct BungeeCord {}

impl Downloader for BungeeCord {
    fn get_name(&self) -> String {
        "BungeeCord".to_string()
    }

    fn get_description(&self) -> String {
        "A server that supports BungeeCord plugins.".to_string()
    }

    fn get_type(&self) -> ServerType {
        Proxy
    }

    async fn install(client: reqwest::Client, _minecraft_version: Option<String>) -> Result<String, crate::downloaderror::DownloadError> {
        download_file(&client, "https://ci.md-5.net/job/BungeeCord/lastSuccessfulBuild/artifact/bootstrap/target/BungeeCord.jar", "./server.jar").await?;

        Ok("".to_string())
    }

    async fn startup_message(_string: &String) -> Option<std::net::SocketAddrV4> {
        basic_proxy_address_from_string(_string).await
    }
}