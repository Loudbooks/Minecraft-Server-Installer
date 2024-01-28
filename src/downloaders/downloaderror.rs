use reqwest::Error;

#[derive(Debug)]
pub enum DownloadError {
    Success,
    Failure,
}

impl From<Error> for DownloadError {
    fn from(_: Error) -> Self {
        DownloadError::Failure
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(_: std::io::Error) -> Self {
        DownloadError::Failure
    }
}