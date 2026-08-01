use crate::anime::{AnimeRepository, AnimeRepositoryError, CachedAnimeRepository};
use crate::jellyfin::config::SeriesCategory;
use crate::jellyfin::series::{ResolvedSeriesMetadata, SeriesMetadataResolver};
use log::debug;
use nenechi_myanimelist::{AnimeDetails, MediaType, RelationType};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimeRecord {
    pub id: u64,
    pub title: String,
    pub alternative_titles: Vec<String>,
    pub media_type: String,
    pub start_date: Option<String>,
    pub prequel_ids: Vec<u64>,
    pub sequel_ids: Vec<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnimeSeason {
    pub anime_id: u64,
    pub title: String,
    pub season: u32,
    pub start_date: Option<String>,
}

impl From<AnimeDetails> for AnimeRecord {
    fn from(anime: AnimeDetails) -> Self {
        let mut alternative_titles = anime.alternative_titles.synonyms;
        alternative_titles.extend(anime.alternative_titles.en);
        alternative_titles.extend(anime.alternative_titles.ja);

        let mut prequel_ids = Vec::new();
        let mut sequel_ids = Vec::new();
        for related in anime.related_anime {
            match related.relation_type {
                RelationType::Prequel => prequel_ids.push(related.node.id),
                RelationType::Sequel => sequel_ids.push(related.node.id),
                _ => {}
            }
        }

        Self {
            id: anime.id,
            title: anime.title,
            alternative_titles,
            media_type: anime.media_type.to_string(),
            start_date: anime.start_date,
            prequel_ids,
            sequel_ids,
        }
    }
}

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

struct AnimeSeasonResolver<R>
where
    R: AnimeRepository,
{
    repository: R,
}

impl<R> AnimeSeasonResolver<R>
where
    R: AnimeRepository,
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

pub struct AnimeResolver<R>
where
    R: AnimeRepository,
{
    season_resolver: AnimeSeasonResolver<R>,
    title_resolver: AnimeTitleResolver<R>,
}

impl AnimeResolver<CachedAnimeRepository> {
    pub fn build() -> Result<Self, AnimeResolverError> {
        Ok(Self::new(CachedAnimeRepository::get_instance()?))
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
        }
    }

    pub fn resolve_by_title(
        &self,
        title: &str,
    ) -> Result<ResolvedSeriesMetadata, AnimeResolverError> {
        let anime = self.title_resolver.resolve(title)?;
        if anime.is_none() {
            return Err(AnimeResolverError::AnimeNotFound(format!(
                "{} no resuelto, puede ser que no haya coincidido en la busqueda",
                title
            )));
        }

        let anime = anime.unwrap();
        let season = self.season_resolver.resolve(anime.id)?;
        if season.is_none() {
            return Err(AnimeResolverError::SeasonNotFound);
        }

        Ok(ResolvedSeriesMetadata {
            title: anime.title,
            season: season.unwrap(),
        })
    }
}

impl<R> SeriesMetadataResolver for AnimeResolver<R>
where
    R: AnimeRepository + Clone,
{
    fn resolve(
        &self,
        source_directory: &Path,
        category: &SeriesCategory,
    ) -> Result<ResolvedSeriesMetadata, Box<dyn Error>> {
        if category != &SeriesCategory::Anime {
            return Err(Box::new(AnimeResolverError::NotAnimeCategory));
        }
        let title = source_directory
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(AnimeResolverError::EmptyTitle)?;

        self.resolve_by_title(title)
            .map_err(|error| Box::new(error) as Box<dyn Error>)
    }
}

fn same_title(left: &str, right: &str) -> bool {
    normalize_title(left) == normalize_title(right)
}

fn normalize_title(title: &str) -> String {
    title
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
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
    fn anime_title_resolver_matches_an_alternative_title() {
        let resolver = AnimeTitleResolver::new(FakeAnimeRepository::new());

        let anime = resolver
            .resolve("KonoSuba God's Blessing on This Wonderful World 4")
            .unwrap()
            .unwrap();

        assert_eq!(anime.id, 61203);
        assert_eq!(anime.title, "Kono Subarashii Sekai ni Shukufuku wo! 4");
    }

    #[test]
    fn anime_resolver_resolves_the_canonical_title_and_season() {
        let resolver = AnimeResolver::new(FakeAnimeRepository::new());

        let metadata = resolver
            .resolve_by_title("KonoSuba God's Blessing on This Wonderful World 4")
            .unwrap();

        assert_eq!(
            metadata,
            ResolvedSeriesMetadata {
                title: "Kono Subarashii Sekai ni Shukufuku wo! 4".into(),
                season: 4,
            }
        );
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
