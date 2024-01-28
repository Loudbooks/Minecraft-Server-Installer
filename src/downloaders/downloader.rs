use std::cmp::min;
use std::fs::File;
use std::io::Write;
use async_trait::async_trait;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use crate::downloaders::downloaderror::DownloadError;

#[async_trait]
pub trait Downloader {
    async fn download(client: &Client) -> Result<(), DownloadError>;
}

pub async fn download_file(client: &Client, url: &str, path: &str) -> Result<(), DownloadError> {
    let request = client.get(url).send().await?;
    let total_size = request.content_length().unwrap();

    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
        .progress_chars("#>-"));

    let mut file = File::create(path)?;
    let mut download_progress: u64 = 0;
    let mut stream = request.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item?;
        file.write_all(&chunk)?;

        let new = min(download_progress + (chunk.len() as u64), total_size);

        download_progress = new;
        progress_bar.set_position(new);
    }

    progress_bar.finish_with_message(format!("Downloaded {} to {}.", url, path));

    return Ok(());
}