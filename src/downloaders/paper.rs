use reqwest::Client;
use crate::downloaders::downloader::{download_file, Downloader};
use crate::downloaders::downloaderror::DownloadError;

pub(crate) struct Paper {}

impl Downloader for Paper {
    async fn download(client: Client) -> Result<(), DownloadError> {
        let paper_version = get_latest_paper_version().await.unwrap();
        let latest_build = get_latest_build(&paper_version).await.unwrap();

        println!("Using Paper version {} with build {}.", paper_version, latest_build);

        let url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds/{}/downloads/{}", paper_version, latest_build,
                          format!("paper-{}-{}.jar", paper_version, latest_build));

        download_file(
            &client,
            &url,
            "./server.jar"
        ).await?;

        return Ok(());
    }
}

async fn get_latest_paper_version() -> Result<String, String> {
    let url = "https://papermc.io/api/v2/projects/paper";
    let response = reqwest::get(url).await.unwrap();
    let json: serde_json::Value = response.json().await.unwrap();
    let versions = json["versions"].as_array().ok_or("JSON is invalid!")?;
    let paper_version = versions.last().and_then(|v| v.as_str()).ok_or("Paper version not found!")?;

    return Ok(paper_version.to_string());
}

async fn get_latest_build(paper_version: &str) -> Result<String, String> {
    let url = format!("https://api.papermc.io/v2/projects/paper/versions/{}/builds", paper_version);
    let response = reqwest::get(&url).await.unwrap();
    let json: serde_json::Value = response.json().await.unwrap();

    let build = json["builds"]
        .as_array()
        .ok_or("JSON is invalid")?
        .iter()
        .filter_map(|build| {
            let channel = build["channel"].as_str()?;
            let build_number = build["build"].as_u64()?;
            if channel == "default" {
                Some(build_number)
            } else {
                None
            }
        })
        .max()
        .ok_or("No builds found")?;

    return Ok(build.to_string())
}