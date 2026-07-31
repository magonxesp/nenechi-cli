use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::path::Path;

use nenechi_myanimelist::{Anime, AnimeDetails, Client, RelationType};

use crate::jellyfin::config::SeriesCategory;
use crate::jellyfin::series::{ResolvedSeriesMetadata, SeriesMetadataResolver};

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
            media_type: anime.media_type,
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
    AmbiguousAnimeTitle {
        title: String,
        candidates: Vec<String>,
    },
    NotAnimeCategory,
    RelationCycle(u64),
    AnimeOutsideMainSequence(u64),
    Catalog(String),
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
            Self::AmbiguousAnimeTitle { title, candidates } => write!(
                formatter,
                "MyAnimeList devolvió varias entradas para {title:?} sin una coincidencia exacta: {}",
                candidates.join(", ")
            ),
            Self::NotAnimeCategory => {
                formatter.write_str("el resolver de anime solo admite la categoría anime")
            }
            Self::RelationCycle(anime_id) => write!(
                formatter,
                "se detectó un ciclo en las relaciones de MyAnimeList para el anime {anime_id}"
            ),
            Self::AnimeOutsideMainSequence(anime_id) => write!(
                formatter,
                "el anime {anime_id} no pertenece a la secuencia principal de secuelas"
            ),
            Self::Catalog(error) => write!(formatter, "error consultando MyAnimeList: {error}"),
        }
    }
}

impl Error for AnimeResolverError {}

pub struct AnimeResolver {
    client: Client,
}

impl AnimeResolver {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn resolve_title(&self, title: &str) -> Result<ResolvedSeriesMetadata, AnimeResolverError> {
        let (target_id, seasons) = self.resolve_seasons(title)?;
        let target = seasons
            .iter()
            .find(|anime| anime.anime_id == target_id)
            .ok_or(AnimeResolverError::AnimeOutsideMainSequence(target_id))?;
        let series_title = seasons
            .first()
            .expect("resolve_seasons siempre devuelve al menos una temporada")
            .title
            .clone();

        Ok(ResolvedSeriesMetadata {
            title: series_title,
            season: target.season,
        })
    }

    pub fn seasons(&self, title: &str) -> Result<Vec<AnimeSeason>, AnimeResolverError> {
        self.resolve_seasons(title).map(|(_, seasons)| seasons)
    }

    fn resolve_seasons(&self, title: &str) -> Result<(u64, Vec<AnimeSeason>), AnimeResolverError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(AnimeResolverError::EmptyTitle);
        }

        let search_response = self
            .client
            .search_anime(title)
            .map_err(|error| AnimeResolverError::Catalog(error.to_string()))?;
        let search_results = search_response
            .data
            .into_iter()
            .map(|entry| entry.node)
            .collect::<Vec<_>>();
        if search_results.is_empty() {
            return Err(AnimeResolverError::AnimeNotFound(title.into()));
        }

        let mut cache = HashMap::new();
        let target = self.select_search_result(title, search_results, &mut cache)?;
        self.resolve_target(target, &mut cache)
    }

    fn resolve_target(
        &self,
        target: AnimeRecord,
        cache: &mut HashMap<u64, AnimeRecord>,
    ) -> Result<(u64, Vec<AnimeSeason>), AnimeResolverError> {
        let target_id = target.id;
        let root = self.find_root(target, cache)?;
        let seasons = self.follow_sequels(root, cache)?;

        if !seasons.iter().any(|anime| anime.anime_id == target_id) {
            return Err(AnimeResolverError::AnimeOutsideMainSequence(target_id));
        }

        Ok((target_id, seasons))
    }

    fn select_search_result(
        &self,
        query: &str,
        mut results: Vec<Anime>,
        cache: &mut HashMap<u64, AnimeRecord>,
    ) -> Result<AnimeRecord, AnimeResolverError> {
        results.sort_by_key(|result| !same_title(&result.title, query));
        let mut candidates = Vec::new();

        for result in results {
            let anime = self.get_cached(result.id, cache)?;
            if same_title(&anime.title, query)
                || anime
                    .alternative_titles
                    .iter()
                    .any(|title| same_title(title, query))
            {
                return Ok(anime);
            }
            candidates.push(anime);
        }

        match candidates.len() {
            1 => Ok(candidates.pop().unwrap()),
            _ => Err(AnimeResolverError::AmbiguousAnimeTitle {
                title: query.into(),
                candidates: candidates
                    .into_iter()
                    .map(|anime| format!("{} ({})", anime.title, anime.id))
                    .collect(),
            }),
        }
    }

    fn find_root(
        &self,
        mut anime: AnimeRecord,
        cache: &mut HashMap<u64, AnimeRecord>,
    ) -> Result<AnimeRecord, AnimeResolverError> {
        let mut visited = HashSet::new();

        loop {
            if !visited.insert(anime.id) {
                return Err(AnimeResolverError::RelationCycle(anime.id));
            }

            let candidates = self.relations(&anime.prequel_ids, cache)?;
            let Some(prequel) = select_previous(&anime, candidates) else {
                return Ok(anime);
            };
            anime = prequel;
        }
    }

    fn follow_sequels(
        &self,
        mut anime: AnimeRecord,
        cache: &mut HashMap<u64, AnimeRecord>,
    ) -> Result<Vec<AnimeSeason>, AnimeResolverError> {
        let mut visited = HashSet::new();
        let mut seasons = Vec::new();

        loop {
            if !visited.insert(anime.id) {
                return Err(AnimeResolverError::RelationCycle(anime.id));
            }

            seasons.push(AnimeSeason {
                anime_id: anime.id,
                title: anime.title.clone(),
                season: seasons.len() as u32 + 1,
                start_date: anime.start_date.clone(),
            });

            let candidates = self.relations(&anime.sequel_ids, cache)?;
            let Some(sequel) = select_next(&anime, candidates) else {
                return Ok(seasons);
            };
            anime = sequel;
        }
    }

    fn relations(
        &self,
        anime_ids: &[u64],
        cache: &mut HashMap<u64, AnimeRecord>,
    ) -> Result<Vec<AnimeRecord>, AnimeResolverError> {
        let mut seen = HashSet::new();
        let mut anime = Vec::new();

        for anime_id in anime_ids {
            if !seen.insert(*anime_id) {
                continue;
            }
            let related = self.get_cached(*anime_id, cache)?;
            anime.push(related);
        }

        Ok(anime)
    }

    fn get_cached(
        &self,
        anime_id: u64,
        cache: &mut HashMap<u64, AnimeRecord>,
    ) -> Result<AnimeRecord, AnimeResolverError> {
        if let Some(anime) = cache.get(&anime_id) {
            return Ok(anime.clone());
        }

        let anime = self
            .client
            .get_anime(anime_id)
            .map_err(|error| AnimeResolverError::Catalog(error.to_string()))?;
        let anime: AnimeRecord = anime.into();
        cache.insert(anime_id, anime.clone());
        Ok(anime)
    }
}

impl SeriesMetadataResolver for AnimeResolver {
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

        self.resolve_title(title)
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

fn select_previous(current: &AnimeRecord, candidates: Vec<AnimeRecord>) -> Option<AnimeRecord> {
    select_by_date(current, candidates, Direction::Previous)
}

fn select_next(current: &AnimeRecord, candidates: Vec<AnimeRecord>) -> Option<AnimeRecord> {
    select_by_date(current, candidates, Direction::Next)
}

enum Direction {
    Previous,
    Next,
}

fn select_by_date(
    current: &AnimeRecord,
    candidates: Vec<AnimeRecord>,
    direction: Direction,
) -> Option<AnimeRecord> {
    let mut dated: Vec<_> = candidates
        .iter()
        .filter(|anime| anime.start_date.is_some())
        .collect();

    if let Some(current_date) = current.start_date.as_deref() {
        let chronological: Vec<_> = dated
            .iter()
            .copied()
            .filter(|anime| match direction {
                Direction::Previous => anime.start_date.as_deref() <= Some(current_date),
                Direction::Next => anime.start_date.as_deref() >= Some(current_date),
            })
            .collect();
        if !chronological.is_empty() {
            dated = chronological;
        }
    }

    dated.sort_by(|left, right| {
        left.start_date
            .cmp(&right.start_date)
            .then(left.id.cmp(&right.id))
    });
    let selected = match direction {
        Direction::Previous => dated.last(),
        Direction::Next => dated.first(),
    };

    selected
        .cloned()
        .cloned()
        .or_else(|| candidates.into_iter().min_by_key(|anime| anime.id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anime(
        id: u64,
        title: &str,
        media_type: &str,
        start_date: &str,
        prequel_ids: &[u64],
        sequel_ids: &[u64],
    ) -> AnimeRecord {
        AnimeRecord {
            id,
            title: title.into(),
            alternative_titles: Vec::new(),
            media_type: media_type.into(),
            start_date: Some(start_date.into()),
            prequel_ids: prequel_ids.into(),
            sequel_ids: sequel_ids.into(),
        }
    }

    fn resolver(anime: Vec<AnimeRecord>) -> (AnimeResolver, HashMap<u64, AnimeRecord>) {
        (
            AnimeResolver::new(Client::new("test-api-key").unwrap()),
            anime.into_iter().map(|entry| (entry.id, entry)).collect(),
        )
    }

    fn resolve_from_cache(
        resolver: &AnimeResolver,
        target_id: u64,
        cache: &mut HashMap<u64, AnimeRecord>,
    ) -> Result<(u64, Vec<AnimeSeason>), AnimeResolverError> {
        let target = cache.get(&target_id).unwrap().clone();
        resolver.resolve_target(target, cache)
    }

    #[test]
    fn resolves_the_root_title_and_target_season() {
        let (resolver, mut cache) = resolver(vec![
            anime(1, "Example", "tv", "2020-01-01", &[], &[2]),
            anime(2, "Example 2", "tv", "2021-01-01", &[1], &[3]),
            anime(3, "Example 3", "tv", "2022-01-01", &[2], &[]),
        ]);

        let (target_id, seasons) = resolve_from_cache(&resolver, 2, &mut cache).unwrap();
        let target = seasons
            .iter()
            .find(|season| season.anime_id == target_id)
            .unwrap();
        assert_eq!(seasons[0].title, "Example");
        assert_eq!(target.season, 2);
    }

    #[test]
    fn follows_related_anime_regardless_of_media_type() {
        let (resolver, mut cache) = resolver(vec![
            anime(1, "Example Movie", "movie", "2019-01-01", &[], &[2]),
            anime(2, "Example", "tv", "2020-01-01", &[1], &[3]),
            anime(3, "Example OVA", "ova", "2020-06-01", &[2], &[4]),
            anime(4, "Example ONA", "ona", "2021-01-01", &[3], &[]),
        ]);

        let (_, seasons) = resolve_from_cache(&resolver, 2, &mut cache).unwrap();
        assert_eq!(
            seasons
                .iter()
                .map(|season| season.anime_id)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4]
        );
    }

    #[test]
    fn selects_an_exact_movie_title() {
        let (resolver, mut cache) = resolver(vec![anime(
            1,
            "Example Movie",
            "movie",
            "2020-01-01",
            &[],
            &[],
        )]);
        let results = vec![Anime {
            id: 1,
            title: "Example Movie".into(),
            main_picture: None,
        }];

        let selected = resolver
            .select_search_result("example movie", results, &mut cache)
            .unwrap();

        assert_eq!(selected.id, 1);
    }

    #[test]
    fn chooses_the_closest_later_sequel_by_air_date() {
        let (resolver, mut cache) = resolver(vec![
            anime(1, "Example", "tv", "2020-01-01", &[], &[3, 2]),
            anime(2, "Example 2", "tv", "2021-01-01", &[1], &[3]),
            anime(3, "Example 3", "tv", "2022-01-01", &[1, 2], &[]),
        ]);

        let (_, seasons) = resolve_from_cache(&resolver, 1, &mut cache).unwrap();
        assert_eq!(
            seasons
                .iter()
                .map(|season| (season.anime_id, season.season))
                .collect::<Vec<_>>(),
            vec![(1, 1), (2, 2), (3, 3)]
        );
    }

    #[test]
    fn detects_relation_cycles() {
        let (resolver, mut cache) = resolver(vec![
            anime(1, "Example", "tv", "2020-01-01", &[], &[2]),
            anime(2, "Example 2", "tv", "2021-01-01", &[1], &[1]),
        ]);

        assert_eq!(
            resolve_from_cache(&resolver, 1, &mut cache),
            Err(AnimeResolverError::RelationCycle(1))
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

    #[test]
    fn selects_the_exact_normalized_title_instead_of_api_order() {
        let (resolver, mut cache) = resolver(vec![
            anime(
                2,
                "Example: Long Title 2nd Season",
                "tv",
                "2021-01-01",
                &[],
                &[],
            ),
            anime(
                3,
                "Example: Long Title 3rd Season",
                "tv",
                "2022-01-01",
                &[],
                &[],
            ),
        ]);
        let results = vec![
            Anime {
                id: 2,
                title: "Example: Long Title 2nd Season".into(),
                main_picture: None,
            },
            Anime {
                id: 3,
                title: "Example: Long Title 3rd Season".into(),
                main_picture: None,
            },
        ];

        let selected = resolver
            .select_search_result("example long title 3rd season", results, &mut cache)
            .unwrap();

        assert_eq!(selected.id, 3);
    }

    #[test]
    fn rejects_ambiguous_search_results_without_an_exact_title() {
        let (resolver, mut cache) = resolver(vec![
            anime(2, "Example 2nd Season", "tv", "2021-01-01", &[], &[]),
            anime(3, "Example 3rd Season", "tv", "2022-01-01", &[], &[]),
        ]);
        let results = vec![
            Anime {
                id: 2,
                title: "Example 2nd Season".into(),
                main_picture: None,
            },
            Anime {
                id: 3,
                title: "Example 3rd Season".into(),
                main_picture: None,
            },
        ];

        let error = resolver
            .select_search_result("Example", results, &mut cache)
            .unwrap_err();

        assert_eq!(
            error,
            AnimeResolverError::AmbiguousAnimeTitle {
                title: "Example".into(),
                candidates: vec![
                    "Example 2nd Season (2)".into(),
                    "Example 3rd Season (3)".into(),
                ],
            }
        );
    }
}
