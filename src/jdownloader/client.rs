use crate::config::CliConfig;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Serialize)]
struct AddLinksQuery {
    links: String,
    autostart: bool,
    #[serde(rename = "packageName")]
    package_name: String,
    #[serde(rename = "destinationFolder")]
    destination_folder: String,
    #[serde(rename = "assignJobID")]
    assign_job_id: bool,
}

pub type JobId = i64;

#[derive(Deserialize)]
struct LinkCollectingJob {
    id: JobId,
}

#[derive(Deserialize)]
struct AddLinksResponse {
    data: LinkCollectingJob
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueryLinks {
    #[serde(rename = "jobUUIDs")]
    job_uuids: Vec<i64>,
    name: bool,
    bytes_loaded: bool,
    bytes_total: bool,
    speed: bool,
    status: bool,
    finished: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadLink {
    pub uuid: i64,
    pub name: String,
    #[serde(default)]
    pub bytes_loaded: i64,
    #[serde(default)]
    pub bytes_total: i64,
    #[serde(default)]
    pub speed: i64,
    pub status: Option<String>,
    #[serde(default)]
    pub finished: bool,
}

#[derive(Deserialize)]
struct QueryLinksResponse {
    data: Vec<DownloadLink>
}

static JDOWNLOADER_INSTANCE: OnceLock<JDownloader> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct JDownloader {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl JDownloader {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            base_url: base_url.into(),
        }
    }

    pub fn from_config(config: &CliConfig) -> Self {
        Self::new(config.jdownloader.api_base_url.trim().trim_end_matches("/"))
    }

    pub fn get_instance<'a>() -> &'a Self {
        JDOWNLOADER_INSTANCE.get_or_init(|| {
            let config = CliConfig::get_instance();
            JDownloader::from_config(&config)
        })
    }

    pub fn download(
        &self,
        urls: &Vec<String>,
        destination: &PathBuf,
        package_name: &String
    ) -> Result<JobId, JDownloaderError> {
        let query = AddLinksQuery {
            links: urls.join("\n"),
            autostart: true,
            package_name: package_name.clone(),
            destination_folder: destination.to_string_lossy().to_string(),
            assign_job_id: true,
        };

        let query = serde_json::to_string(&query)?;
        let url = format!("{}/linkgrabberv2/addLinks", self.base_url);

        let response = self.client
            .get(&url)
            .query(&[("query", query)])
            .send()?
            .error_for_status()?
            .json::<AddLinksResponse>()?;

        Ok(response.data.id)
    }

    pub fn check_progress(&self, job_id: JobId) -> Result<Vec<DownloadLink>, JDownloaderError> {
        let query = QueryLinks {
            job_uuids: vec![job_id],
            name: true,
            bytes_loaded: true,
            bytes_total: true,
            speed: true,
            status: true,
            finished: true,
        };

        let url = format!("{}/downloadsV2/queryLinks", self.base_url);
        let links = self.client
            .get(&url)
            .query(&[(
                "queryParams",
                serde_json::to_string(&query)?,
            )])
            .send()?
            .error_for_status()?
            .json::<QueryLinksResponse>()?;

        Ok(links.data)
    }
}

#[derive(Debug)]
pub enum JDownloaderError {
    HTTP(String),
    Client(String)
}

impl Display for JDownloaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JDownloaderError::HTTP(s) => write!(f, "http error: {}", s),
            JDownloaderError::Client(s) => write!(f, "client error: {}", s)
        }
    }
}

impl std::error::Error for JDownloaderError {}

impl From<reqwest::Error> for JDownloaderError {
    fn from(e: reqwest::Error) -> Self {
        JDownloaderError::HTTP(e.to_string())
    }
}

impl From<serde_json::Error> for JDownloaderError {
    fn from(e: serde_json::Error) -> Self {
        JDownloaderError::Client(format!("serialization error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_client_from_jdownloader_config() {
        let mut config = CliConfig::default();
        config.jdownloader.api_base_url = "http://jdownloader.test:3128".to_string();

        let client = JDownloader::from_config(&config);

        assert_eq!(client.base_url, "http://jdownloader.test:3128");
    }
}
