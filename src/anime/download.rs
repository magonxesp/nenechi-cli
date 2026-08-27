use crate::anime::CachedAnimeRepository;
use crate::config::CliConfig;
use crate::fs::strip_illegal_chars;
use crate::jdownloader::{JDownloader, JDownloaderError, JobId};
use crate::media::{AnimeResolver, AnimeResolverError};
use crate::osaka::{Links, OsakaClient, OsakaError, OsakaResponse};
use std::fmt::Display;
use std::path::PathBuf;
use log::{debug, trace};

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

    pub fn download(&self, url: &str, incremental: bool) -> Result<(), AnimeDownloadError> {
        debug!("extracting links using Osaka: {}", url);
        let links = self.extract_links(url, incremental)?;
        let title = links.title.clone();
        debug!("series title: {}", title);

        let links = Self::resolve_links(&links.links)?;
        let anime_directory = PathBuf::from(&self.config.media.anime.directory);

        debug!("starting download with JDonwloader: {:?}", anime_directory);
        let sanitized_title = strip_illegal_chars(&title);
        let anime_destination = anime_directory.join(&sanitized_title);

        debug!("sanitized package name: {}", sanitized_title);
        let job_id = self.jdownloader.download(&links, &anime_destination, &sanitized_title)?;
        self.wait_finish_downloads(job_id, links)?;

        let downloaded_directory = anime_directory.join(&sanitized_title);
        debug!("fetching and writing metadata: {:?}", downloaded_directory);
        let metadata = self.resolver.resolve_by_title(&title)?;
        metadata.write(&downloaded_directory)
            .map_err(|err| AnimeDownloadError::Metadata(format!("error writing metadata: {}", err)))?;

        Ok(())
    }

    fn wait_finish_downloads(&self, job_id: JobId, links: Vec<String>) -> Result<(), AnimeDownloadError> {
        debug!("waiting for download finish: {}", job_id);

        loop {
            let progress = self.jdownloader.check_progress(job_id)?;
            if progress.is_empty() {
                debug!("downloads haven't started yet, waiting 3 seconds to next check: {}", job_id);
                std::thread::sleep(std::time::Duration::from_secs(3));
                continue;
            }

            let mut finished = 0;

            for item in progress {
                if item.finished {
                    finished += 1;
                }
            }

            debug!("downloads finished {} of {}", finished, links.len());
            if finished == links.len() {
                debug!("download finished: {}", job_id);
                return Ok(());
            }

            debug!("waiting 3 seconds for check download status: {}", job_id);
            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    }

    fn extract_links(&self, url: &str, incremental: bool) -> Result<OsakaResponse, AnimeDownloadError> {
        debug!("extracting links using Osaka: {}", url);

        let links = if incremental {
            let anime_directory = PathBuf::from(&self.config.media.anime.directory);
            let response = self.osaka.extract_links(url, Some(usize::MAX))?;
            let title = response.title.clone();
            let sanitized_title = strip_illegal_chars(&title);
            let anime_destination = anime_directory.join(&sanitized_title);
            let episodes = self.count_episodes(anime_destination.clone())?;
            debug!(
                "extracting pending episodes download links: {}: {:?} ({} episodes)",
                url, anime_destination, episodes
            );
            self.osaka.extract_links(url, Some(episodes + 1))?
        } else {
            debug!("extracting all episodes download links: {}", url);
            self.osaka.extract_links(url, None)?
        };

        Ok(links)
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
        debug!("preferred links: {:?} ({})", links, links.len());

        Ok(links)
    }

    fn count_episodes(&self, path: PathBuf) -> Result<usize, AnimeDownloadError> {
        if !path.exists() {
            debug!("path not exists, counting 0 episodes: {:?}", path);
            return Ok(0);
        }

        debug!("path exists, counting episodes of: {:?}", path);
        let mut episodes = 0;
        let paths = std::fs::read_dir(path)?;
        let patterns = self.config.media.anime.episodes_whitelist.clone();

        for path in paths {
            trace!("evaluating path: {:?}", path);

            if let Ok(path) = path {
                if is_episode(&path.path(), &patterns)? {
                    trace!("counting path as episode: {:?}", path);
                    episodes += 1;
                }
            }
        }

        Ok(episodes)
    }
}

fn is_episode(path: &PathBuf, patterns: &Vec<String>) -> Result<bool, glob::PatternError> {
    debug!("checking path is episode using globs ({:?}): {:?}", patterns, path);

    for pattern in patterns {
        let pattern = glob::Pattern::new(pattern.as_str())?;
        let result = pattern.matches_path(&path);
        trace!("glob result: {:?}: {}", pattern, result);

        if result {
            debug!("glob match: {:?}; on episode: {:?}", pattern, path);
            return Ok(true);
        }
    }

    Ok(false)
}

#[derive(Debug)]
pub enum AnimeDownloadError {
    Osaka(String),
    JDownloader(String),
    LinksNotAvailable(String),
    Metadata(String),
    IO(String),
    Other(String),
}

impl Display for AnimeDownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimeDownloadError::Osaka(err) => write!(f, "osaka error: {}", err),
            AnimeDownloadError::LinksNotAvailable(err) => write!(f, "{}", err),
            AnimeDownloadError::JDownloader(err) => write!(f, "JDownloader error: {}", err),
            AnimeDownloadError::Metadata(err) => write!(f, "metadata error: {}", err),
            AnimeDownloadError::IO(err) => write!(f, "io error: {}", err),
            AnimeDownloadError::Other(err) => write!(f, "{}", err),
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

impl From<std::io::Error> for AnimeDownloadError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value.to_string())
    }
}

impl From<glob::PatternError> for AnimeDownloadError {
    fn from(value: glob::PatternError) -> Self {
        Self::Other(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::anime::download::is_episode;

    #[test]
    fn test_is_episode() {
        let path = PathBuf::from("/path/to/anime/episode_01.mp4");
        let patterns = vec!["*.mp4".to_string()];

        let result = is_episode(&path, &patterns);

        assert!(result.is_ok());
        assert_eq!(true, result.unwrap());
    }
}
