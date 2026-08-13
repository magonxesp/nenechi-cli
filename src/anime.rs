use crate::client::MyAnimeListClient;
use log::debug;
#[cfg(test)]
use nenechi_myanimelist::WebSearchResponse;
use nenechi_myanimelist::{Anime, AnimeDetails, Client, ClientError};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};
use crate::config::CliConfig;

#[derive(Debug)]
pub enum AnimeRepositoryError {
    CreateInstance(String),
    CacheLock(String),
    Other(String),
}

impl From<ClientError> for AnimeRepositoryError {
    fn from(err: ClientError) -> Self {
        AnimeRepositoryError::Other(err.to_string())
    }
}

impl<T> From<PoisonError<T>> for AnimeRepositoryError {
    fn from(err: PoisonError<T>) -> Self {
        AnimeRepositoryError::CacheLock(err.to_string())
    }
}

impl Display for AnimeRepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimeRepositoryError::CreateInstance(err) => {
                write!(f, "create instance error: {}", err)
            }
            AnimeRepositoryError::CacheLock(err) => write!(f, "cache lock error: {}", err),
            AnimeRepositoryError::Other(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for AnimeRepositoryError {}

pub trait AnimeRepository {
    fn find_by_id(&self, anime_id: u64) -> Result<Option<AnimeDetails>, AnimeRepositoryError>;
    fn search(&self, title: &str) -> Result<Vec<Anime>, AnimeRepositoryError>;
}

#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub struct FakeAnimeRepository {
    // search is the optional suffix of myanimelist_web_search.json.
    // For example: k_on will be myanimelist_web_search_k_on.json.
    // It allow to mock specific searches
    search: String,
}

#[cfg(test)]
impl FakeAnimeRepository {
    pub fn new() -> Self {
        Self {
            search: String::from(""),
        }
    }

    pub fn with_search(search: &str) -> Self {
        Self { search: search.to_string() }
    }
}

#[cfg(test)]
impl AnimeRepository for FakeAnimeRepository {
    fn find_by_id(&self, anime_id: u64) -> Result<Option<AnimeDetails>, AnimeRepositoryError> {
        let fixture = match anime_id {
            30831 => include_str!("../tests/fixtures/myanimelist/anime_30831.json"),
            32937 => include_str!("../tests/fixtures/myanimelist/anime_32937.json"),
            38040 => include_str!("../tests/fixtures/myanimelist/anime_38040.json"),
            49458 => include_str!("../tests/fixtures/myanimelist/anime_49458.json"),
            61203 => include_str!("../tests/fixtures/myanimelist/anime_61203.json"),
            5680 => include_str!("../tests/fixtures/myanimelist/anime_5680.json"),
            7791 => include_str!("../tests/fixtures/myanimelist/anime_7791.json"),
            _ => return Ok(None),
        };

        serde_json::from_str(fixture)
            .map(Some)
            .map_err(|error| AnimeRepositoryError::Other(error.to_string()))
    }

    fn search(&self, _title: &str) -> Result<Vec<Anime>, AnimeRepositoryError> {
        let fixture: WebSearchResponse = serde_json::from_str(match self.search.as_str() {
            "k_on" => include_str!("../tests/fixtures/myanimelist/web_search_k_on.json"),
            _ => include_str!("../tests/fixtures/myanimelist/web_search.json")
        })
        .map_err(|error| AnimeRepositoryError::Other(error.to_string()))?;

        Ok(fixture.into_anime())
    }
}

static CACHED_ANIME_REPOSITORY: OnceLock<CachedAnimeRepository> = OnceLock::new();

#[derive(Clone)]
pub struct CachedAnimeRepository {
    client: Client,
    cache: Arc<Mutex<HashMap<u64, AnimeDetails>>>,
}

impl CachedAnimeRepository {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn from_config(config: &CliConfig) -> Result<Self, AnimeRepositoryError> {
        let client = MyAnimeListClient::create(&config).map_err(AnimeRepositoryError::CreateInstance)?;
        let repository = CachedAnimeRepository::new(client);
        Ok(repository)
    }

    pub fn get_instance() -> Result<Self, AnimeRepositoryError> {
        if let Some(repository) = CACHED_ANIME_REPOSITORY.get() {
            return Ok(repository.clone());
        }

        let client =
            MyAnimeListClient::get_instance().map_err(AnimeRepositoryError::CreateInstance)?;
        let repository = CachedAnimeRepository::new(client);
        match CACHED_ANIME_REPOSITORY.set(repository.clone()) {
            Ok(_) => Ok(repository),
            Err(actual) => Ok(actual.clone()),
        }
    }
}

impl AnimeRepository for CachedAnimeRepository {
    fn find_by_id(&self, anime_id: u64) -> Result<Option<AnimeDetails>, AnimeRepositoryError> {
        let mut cache = self.cache.lock()?;
        if let Some(cached) = cache.get(&anime_id) {
            debug!("cache hit: {}", anime_id);
            return Ok(Some(cached.clone()));
        }

        if let Some(anime) = self.client.get_by_id(anime_id)? {
            cache.insert(anime_id, anime.clone());
            return Ok(Some(anime));
        }

        Ok(None)
    }

    fn search(&self, title: &str) -> Result<Vec<Anime>, AnimeRepositoryError> {
        Ok(self.client.web_search_by_title(title)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_repository_loads_anime_fixtures_by_id() {
        let repository = FakeAnimeRepository::new();

        for id in [30831, 32937, 38040, 49458, 61203] {
            assert_eq!(repository.find_by_id(id).unwrap().unwrap().id, id);
        }
        assert_eq!(repository.find_by_id(1).unwrap(), None);
    }

    #[test]
    fn fake_repository_loads_the_search_fixture() {
        let repository = FakeAnimeRepository::new();

        let results = repository.search("KonoSuba").unwrap();

        assert_eq!(results.len(), 9);
        assert_eq!(results[0].id, 38040);
        assert!(results.iter().all(|anime| anime.id != 60553));
    }
}
