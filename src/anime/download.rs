use crate::anime::CachedAnimeRepository;
use crate::config::CliConfig;
use crate::fs::strip_illegal_chars;
use crate::jdownloader::{JDownloader, JDownloaderError, JobId};
use crate::media::{AnimeResolver, AnimeResolverError};
use crate::osaka::{Links, OsakaClient, OsakaError};
use std::fmt::Display;
use std::path::PathBuf;
use log::debug;

pub struct AnimeDownloader {
    osaka: OsakaClient,
    jdownloader: JDownloader,
    config: CliConfig,
    resolver: AnimeResolver<CachedAnimeRepository>
}

impl AnimeDownloader {
    pub fn new(
        osaka: &OsakaClient,
        jdownloader: &JDownloader,
        config: &CliConfig,
        resolver: &AnimeResolver<CachedAnimeRepository>,
    ) -> Self {
        Self {
            osaka: osaka.clone(),
            jdownloader: jdownloader.clone(),
            config: config.clone(),
            resolver: resolver.clone()
        }
    }

    pub fn from_config(config: &CliConfig) -> Result<Self, AnimeDownloadError> {
        Ok(Self::new(
            &OsakaClient::from_config(&config)?,
            JDownloader::get_instance(),
            &config,
            &AnimeResolver::from_config(&config)?
        ))
    }

    pub fn download(&self, url: &str) -> Result<(), AnimeDownloadError> {
        debug!("extracting links using Osaka: {}", url);
        let links = self.osaka.extract_links(url)?;
        let title = links.title.clone();
        debug!("series title: {}", title);

        let links = Self::resolve_links(&links.links)?;
        let anime_directory = PathBuf::from(&self.config.media.anime_directory);

        debug!("starting download with JDonwloader: {:?}", anime_directory);
        let sanitized_title = strip_illegal_chars(&title);

        debug!("sanitized package name: {}", sanitized_title);
        let job_id = self.jdownloader.download(&links, &anime_directory, &sanitized_title)?;
        self.wait_finish_downloads(job_id)?;

        let downloaded_directory = anime_directory.join(&sanitized_title);
        debug!("fetching and writing metadata: {:?}", downloaded_directory);
        let metadata = self.resolver.resolve_by_title(&title)?;
        metadata.write(&downloaded_directory)
            .map_err(|err| AnimeDownloadError::Metadata(format!("error writing metadata: {}", err)))?;

        Ok(())
    }

    fn wait_finish_downloads(&self, job_id: JobId) -> Result<(), AnimeDownloadError> {
        debug!("waiting for download finish: {}", job_id);

        loop {
            let progress = self.jdownloader.check_progress(job_id)?;
            let not_finished = progress.iter().any(|download| !download.finished);

            if !not_finished {
                debug!("download finished: {}", job_id);
                return Ok(());
            }

            debug!("waiting 3 seconds for check download status: {}", job_id);
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }

    fn resolve_links(links: &Links) -> Result<Vec<String>, AnimeDownloadError> {
        debug!("resolving preferred download platform links: {:?}", links);
        let links = links.mega.clone()
            .or(links.pdrain.clone())
            .or(links.mp4upload.clone())
            .or(links.n1fichier.clone())
            .or(links.transfer_it.clone());

        if links.is_none() {
            return Err(AnimeDownloadError::LinksNotAvailable("no download provider found".to_string()));
        }

        let links = links.unwrap();
        let links = links.sub;

        if links.is_none() {
            return Err(AnimeDownloadError::LinksNotAvailable("no SUB links found".to_string()));
        }

        let links = links.unwrap();
        debug!("preferred links: {:?}", links);

        Ok(links)
    }
}

#[derive(Debug)]
pub enum AnimeDownloadError {
    Osaka(String),
    JDownloader(String),
    LinksNotAvailable(String),
    Metadata(String),
}

impl Display for AnimeDownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimeDownloadError::Osaka(err) => write!(f, "osaka error: {}", err),
            AnimeDownloadError::LinksNotAvailable(err) => write!(f, "{}", err),
            AnimeDownloadError::JDownloader(err) => write!(f, "JDownloader error: {}", err),
            AnimeDownloadError::Metadata(err) => write!(f, "metadata error: {}", err),
        }
    }
}

impl std::error::Error for AnimeDownloadError {}

impl From<OsakaError> for AnimeDownloadError {
    fn from(value: OsakaError) -> Self {
        Self::Osaka(format!("error extracting links, maybe url is not supported: {}", value.to_string()))
    }
}

impl From<JDownloaderError> for AnimeDownloadError {
    fn from(value: JDownloaderError) -> Self {
        Self::JDownloader(value.to_string())
    }
}

impl From<AnimeResolverError> for AnimeDownloadError {
    fn from(value: AnimeResolverError) -> Self {
        Self::Metadata(value.to_string())
    }
}
