use crate::config::CliConfig;
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::time::Duration;

#[derive(Default, Debug, Clone, Deserialize)]
pub struct OsakaResponse {
    pub title: String,
    pub links: Links
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Links {
    #[serde(rename = "PDrain")]
    pub pdrain: Option<DownloadLinks>,
    #[serde(rename = "Mega")]
    pub mega: Option<DownloadLinks>,
    #[serde(rename = "MP4Upload")]
    pub mp4upload: Option<DownloadLinks>,
    #[serde(rename = "1Fichier")]
    pub n1fichier: Option<DownloadLinks>,
    #[serde(rename = "TransferIt")]
    pub transfer_it: Option<DownloadLinks>,
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadLinks {
    #[serde(rename = "SUB")]
    pub sub: Option<Vec<String>>,
    #[serde(rename = "DUB")]
    pub dub: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct OsakaClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl OsakaClient {
    pub fn new(base_url: &String) -> Result<Self, OsakaError> {
        Ok(Self {
            base_url: base_url.trim_end_matches("/").to_string(),
            client: reqwest::blocking::Client::builder()
                .connect_timeout(Duration::from_secs(10))
                .timeout(None)
                .build()?,
        })
    }

    pub fn from_config(config: &CliConfig) -> Result<Self, OsakaError> {
        Self::new(&config.osaka.base_url)
    }

    pub fn extract_links(&self, url: &str) -> Result<OsakaResponse, OsakaError> {
        let links = self.client
            .get(format!("{}/links", self.base_url))
            .query(&[("url", url)])
            .send()?
            .error_for_status()?
            .json::<OsakaResponse>()?;

        Ok(links)
    }
}

#[derive(Debug)]
pub enum OsakaError {
    Client(String)
}

impl Display for OsakaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OsakaError::Client(e) => write!(f, "client error: {}", e),
        }
    }
}

impl std::error::Error for OsakaError {}

impl From<reqwest::Error> for OsakaError {
    fn from(e: reqwest::Error) -> Self {
        OsakaError::Client(e.to_string())
    }
}
