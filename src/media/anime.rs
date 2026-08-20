use crate::anime::{AnimeRepository, AnimeRepositoryError, CachedAnimeRepository};
use crate::config::CliConfig;
use crate::fs::strip_illegal_chars;
use crate::jellyfin::config::SeriesCategory;
use crate::media::{fetch_image_from_url, SeriesMetadata, SeriesMetadataResolver, MetadataProviderIds, Image, SeriesMetadataResolverError};
use log::{debug, warn};
use nenechi_myanimelist::{AnimeDetails, MediaType, RelationType};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub enum AnimeResolverError {
    EmptyTitle,
    AnimeNotFound(String),
    SeasonNotFound,
    NotAnimeCategory,
    Other(String),
}

impl Display for AnimeResolverError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => formatter.write_str("el título del anime no puede estar vacío"),
            Self::AnimeNotFound(title) => {
                write!(
                    formatter,
                    "MyAnimeList no encontró resultados para {title:?}"
                )
            }
            Self::NotAnimeCategory => {
                formatter.write_str("el resolver de anime solo admite la categoría anime")
            }
            Self::Other(error) => write!(formatter, "{error}"),
            Self::SeasonNotFound => write!(formatter, "season not found"),
        }
    }
}

impl Error for AnimeResolverError {}

impl From<AnimeRepositoryError> for AnimeResolverError {
    fn from(error: AnimeRepositoryError) -> Self {
        AnimeResolverError::Other(error.to_string())
    }
}

impl From<AnimeResolverError> for SeriesMetadataResolverError {
    fn from(value: AnimeResolverError) -> Self {
        match value {
            AnimeResolverError::EmptyTitle => SeriesMetadataResolverError::EmptyTitle,
            AnimeResolverError::AnimeNotFound(_) => SeriesMetadataResolverError::NotFound,
            _ => SeriesMetadataResolverError::Other(value.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
struct AnimeSeasonResolver<R>
where
    R: AnimeRepository + Clone,
{
    repository: R,
}

impl<R> AnimeSeasonResolver<R>
where
    R: AnimeRepository + Clone,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn resolve(&self, anime_id: u64) -> Result<Option<i64>, AnimeResolverError> {
        let mut season = 1;
        let mut current_anime_id = anime_id;

        while let Some(prequel) = self.resolve_prequel(current_anime_id)? {
            if prequel.media_type == MediaType::Tv {
                season += 1;
            }
            current_anime_id = prequel.id;
        }

        Ok(Some(season))
    }

    pub fn resolve_first_season(
        &self,
        anime_id: u64,
    ) -> Result<Option<AnimeDetails>, AnimeResolverError> {
        // first ensure this anime is not the first season, so it shouldn't have a prequel
        if let None = self.resolve_prequel(anime_id)? {
            return Ok(self.repository.find_by_id(anime_id)?);
        }

        let mut next_anime_id = anime_id;
        let mut first_season: Option<AnimeDetails> = None;

        while let Some(prequel) = self.resolve_prequel(next_anime_id)? {
            if prequel.media_type == MediaType::Tv {
                first_season = Some(prequel.clone());
            }

            next_anime_id = prequel.id;
        }

        Ok(first_season)
    }

    fn resolve_prequel(&self, anime_id: u64) -> Result<Option<AnimeDetails>, AnimeResolverError> {
        let anime = match self.repository.find_by_id(anime_id)? {
            Some(anime) => anime,
            None => return Ok(None),
        };

        for related in &anime.related_anime {
            if related.relation_type == RelationType::Prequel {
                if let Some(details) = self.repository.find_by_id(related.node.id)? {
                    return Ok(Some(details.clone()));
                }
            }
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
struct AnimeTitleResolver<R>
where
    R: AnimeRepository,
{
    repository: R,
}

impl<R> AnimeTitleResolver<R>
where
    R: AnimeRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn resolve(&self, title: &str) -> Result<Option<AnimeDetails>, AnimeResolverError> {
        let search_results = self.repository.search(title)?;

        for result in search_results {
            let details = self.repository.find_by_id(result.id)?;
            if details.is_none() {
                continue;
            }

            let details = details.unwrap();
            let mut titles: Vec<&str> = Vec::new();

            titles.push(&details.title);
            if let Some(en) = &details.alternative_titles.en {
                titles.push(en);
            }

            if let Some(ja) = &details.alternative_titles.ja {
                titles.push(ja);
            }

            for synonym in &details.alternative_titles.synonyms {
                titles.push(synonym);
            }

            for &found_title in &titles {
                if same_title(found_title, title) {
                    debug!(
                        "{} coincide con {}, devolviendo los detalles de {}",
                        title, found_title, details.id
                    );

                    return Ok(Some(details.clone()));
                }
            }

            debug!(
                "{:?} no coincide con uno de los items de la busqueda: {:?}",
                title, titles
            );
        }

        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct AnimeResolver<R>
where
    R: AnimeRepository + Clone,
{
    season_resolver: AnimeSeasonResolver<R>,
    title_resolver: AnimeTitleResolver<R>,
    repository: R,
}

impl AnimeResolver<CachedAnimeRepository> {
    pub fn build() -> Result<Self, AnimeResolverError> {
        Ok(Self::new(CachedAnimeRepository::get_instance()?))
    }

    pub fn from_config(config: &CliConfig) -> Result<Self, AnimeResolverError> {
        Ok(Self::new(CachedAnimeRepository::from_config(config)?))
    }
}

impl<R> AnimeResolver<R>
where
    R: AnimeRepository + Clone,
{
    pub fn new(repository: R) -> Self {
        Self {
            season_resolver: AnimeSeasonResolver::new(repository.clone()),
            title_resolver: AnimeTitleResolver::new(repository.clone()),
            repository,
        }
    }

    pub fn resolve_by_title(
        &self,
        title: &str,
    ) -> Result<SeriesMetadata, AnimeResolverError> {
        let anime = self.title_resolver.resolve(title)?;
        if anime.is_none() {
            return Err(AnimeResolverError::AnimeNotFound(format!(
                "{} no resuelto, puede ser que no haya coincidido en la busqueda",
                title
            )));
        }

        let anime = anime.unwrap();
        Ok(self.build_metadata(&anime)?)
    }

    fn build_metadata(&self, anime: &AnimeDetails) -> Result<SeriesMetadata, AnimeResolverError> {
        let season = self.season_resolver.resolve(anime.id)?;
        if season.is_none() {
            return Err(AnimeResolverError::SeasonNotFound);
        }

        let first_season = self.season_resolver.resolve_first_season(anime.id)?;
        if first_season.is_none() {
            warn!("first season not found for: {}", anime.title);
            return Err(AnimeResolverError::SeasonNotFound);
        }

        let first_season = first_season.unwrap();
        let mut metadata: SeriesMetadata = anime.clone().into();
        metadata.title = first_season.title;
        metadata.original_title = first_season.alternative_titles.ja.unwrap_or_default();
        metadata.season = season.unwrap() as u16;
        metadata.cover = Self::fetch_cover(&anime);

        Ok(metadata)
    }

    fn fetch_cover(anime: &AnimeDetails) -> Option<Image> {
        let url = match &anime.main_picture {
            Some(picture) => *&picture.large.as_str(),
            None => return None,
        };

        match fetch_image_from_url(url) {
            Ok(image) => Some(image),
            Err(err) => {
                warn!("failed fetching cover {}: {}", url, err);
                None
            }
        }
    }
}

impl<R> SeriesMetadataResolver for AnimeResolver<R>
where
    R: AnimeRepository + Clone,
{
    fn resolve_from_directory(&self, source_directory: &Path) -> Result<SeriesMetadata, SeriesMetadataResolverError> {
        let title = source_directory
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(AnimeResolverError::EmptyTitle)?;

        Ok(self.resolve_by_title(title)?)
    }

    fn resolve_from_identifier(&self, identifier: &str) -> Result<SeriesMetadata, SeriesMetadataResolverError> {
        let id = identifier.parse::<u64>().map_err(|err| {
            SeriesMetadataResolverError::Other(format!("failed parse id: {}: {}", identifier, err))
        })?;

        let anime = self.repository.find_by_id(id)
            .map_err(|err| SeriesMetadataResolverError::Other(err.to_string()))?;

        if anime.is_none() {
            warn!("no anime found for: {}", identifier);
            return Err(SeriesMetadataResolverError::NotFound);
        }

        let anime = anime.unwrap();
        Ok(self.build_metadata(&anime)?)
    }
}

impl From<AnimeDetails> for SeriesMetadata {
    fn from(details: AnimeDetails) -> Self {
        let year = details
            .start_season
            .as_ref()
            .map(|season| season.year)
            .and_then(|year: u64| u16::try_from(year).ok())
            .unwrap_or_default();

        Self {
            title: details.title.clone(),
            original_title: details.alternative_titles.ja.clone().unwrap_or_default(),
            plot: details.synopsis.unwrap_or_default(),
            year,
            premiered: details.start_date.unwrap_or_default(),
            rating: details.mean.unwrap_or_default() as f32,
            runtime: details
                .average_episode_duration
                .map(|seconds| seconds / 60)
                .and_then(|minutes| u16::try_from(minutes).ok())
                .unwrap_or_default(),
            status: details.status,
            genre: details.genres.into_iter().map(|genre| genre.name).collect(),
            tag: Vec::new(),
            studio: details
                .studios
                .into_iter()
                .next()
                .map(|studio| studio.name)
                .unwrap_or_default(),
            id: MetadataProviderIds {
                mal: Some(details.id.to_string()),
                imdb: None,
                tmdb: None,
            },
            actor: Vec::new(),
            season: 1,
            season_title: Some(details.title),
            season_original_title: details.alternative_titles.ja,
            cover: None,
        }
    }
}

fn same_title(left: &str, right: &str) -> bool {
    normalize_title(left) == normalize_title(right)
}

fn normalize_title(title: &str) -> String {
    strip_illegal_chars(title).to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anime::FakeAnimeRepository;

    #[test]
    fn anime_season_resolver_counts_tv_prequels() {
        let resolver = AnimeSeasonResolver::new(FakeAnimeRepository::new());

        let season = resolver.resolve(61203).unwrap();

        assert_eq!(season, Some(4));
    }

    #[test]
    fn anime_season_resolver_resolve_first_season() {
        let resolver = AnimeSeasonResolver::new(FakeAnimeRepository::new());

        let season = resolver.resolve_first_season(61203).unwrap();

        assert_eq!(season.is_some(), true);
        let season = season.unwrap();

        assert_eq!(season.id, 30831);
    }

    #[test]
    fn anime_title_resolver_matches_an_alternative_title() {
        let resolver = AnimeTitleResolver::new(FakeAnimeRepository::new());

        let anime = resolver
            .resolve("KonoSuba God's Blessing on This Wonderful World! 4")
            .unwrap()
            .unwrap();

        assert_eq!(anime.id, 61203);
        assert_eq!(anime.title, "Kono Subarashii Sekai ni Shukufuku wo! 4");
    }

    #[test]
    fn anime_details_conversion_maps_myanimelist_metadata() {
        let details = FakeAnimeRepository::new()
            .find_by_id(30831)
            .unwrap()
            .unwrap();

        let metadata = SeriesMetadata::from(details);

        assert_eq!(metadata.title, "Kono Subarashii Sekai ni Shukufuku wo!");
        assert_eq!(metadata.original_title, "この素晴らしい世界に祝福を！");
        assert_eq!(metadata.year, 2016);
        assert_eq!(metadata.premiered, "2016-01-14");
        assert_eq!(metadata.rating, 8.09);
        assert_eq!(metadata.runtime, 23);
        assert_eq!(
            metadata.genre,
            ["Adventure", "Comedy", "Fantasy", "Isekai", "Parody"]
        );
        assert_eq!(metadata.studio, "Studio Deen");
        assert_eq!(metadata.id.mal.as_deref(), Some("30831"));
        assert_eq!(metadata.season, 1);
    }

    #[test]
    fn anime_resolver_maps_the_matched_anime() {
        let resolver = AnimeResolver::new(FakeAnimeRepository::new());

        let metadata = resolver
            .resolve_by_title("KonoSuba God's Blessing on This Wonderful World! 4")
            .unwrap();

        assert_eq!(metadata.title, "Kono Subarashii Sekai ni Shukufuku wo!");
        assert_eq!(metadata.season, 4);
    }

    #[test]
    fn anime_resolver_maps_the_matched_anime_k_on_first_season() {
        let resolver = AnimeResolver::new(FakeAnimeRepository::with_search("k_on"));

        let metadata = resolver.resolve_by_title("K-On!").unwrap();

        assert_eq!(metadata.title, "K-On!");
        assert_eq!(metadata.season, 1);
    }

    #[test]
    fn anime_resolver_maps_the_matched_anime_k_on_second_season() {
        let resolver = AnimeResolver::new(FakeAnimeRepository::with_search("k_on"));

        let metadata = resolver.resolve_by_title("K-On!!").unwrap();

        assert_eq!(metadata.title, "K-On!");
        assert_eq!(metadata.season, 2);
    }

    #[test]
    fn anime_resolver_maps_the_matched_anime_k_on_first_season_by_id() {
        let resolver = AnimeResolver::new(FakeAnimeRepository::with_search("k_on"));

        let metadata = resolver.resolve_from_identifier("5680").unwrap();

        assert_eq!(metadata.title, "K-On!");
        assert_eq!(metadata.season, 1);
    }

    #[test]
    fn title_matching_ignores_case_whitespace_and_punctuation() {
        assert!(same_title(
            "Honzuki no Gekokujou: Shisho ni Naru Tame",
            "honzuki no gekokujou shisho ni naru tame",
        ));
    }

    #[test]
    fn title_matching_keeps_seasons_distinct() {
        assert!(!same_title(
            "Honzuki no Gekokujou 2nd Season",
            "Honzuki no Gekokujou 3rd Season",
        ));
    }
}
