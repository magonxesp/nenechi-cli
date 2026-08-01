use crate::jellyfin::anime::AnimeResolver;
use crate::jellyfin::config::{JellyfinConfig, SeriesCategory, TargetConfig, TargetType};
use crate::jellyfin::series::{ResolvedSeriesMetadata, SeriesMetadataResolver};
use crate::jellyfin::series_metadata::{SeriesMetadata, SeriesMetadataRepository};
use crate::jellyfin::{movies, series};
use clap::Subcommand;
use log::{debug, info, warn};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Subcommand)]
pub enum JellyfinCommands {
    Index {
        #[arg(long)]
        force: bool,
    },
    Mount,
}

impl Display for JellyfinCommands {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Index { .. } => formatter.write_str("index"),
            Self::Mount => formatter.write_str("mount"),
        }
    }
}

pub fn execute_jellyfin_command(command: JellyfinCommands) -> Result<(), String> {
    let config = JellyfinConfig::read()?;
    let anime_resolver = anime_resolver(&config)?;
    let result = match command {
        JellyfinCommands::Index { force } => index(
            &config,
            SeriesMetadataRepository::get_instance(),
            anime_resolver
                .as_ref()
                .map(|resolver| resolver as &dyn SeriesMetadataResolver),
            force,
        ),
        JellyfinCommands::Mount => mount(
            &config,
            SeriesMetadataRepository::get_instance(),
            anime_resolver
                .as_ref()
                .map(|resolver| resolver as &dyn SeriesMetadataResolver),
        ),
    };

    result
        .map(|_| ())
        .map_err(|error| format!("subcommand {} failed: {error}", command))
}

fn index(
    config: &JellyfinConfig,
    repository: &SeriesMetadataRepository,
    anime_resolver: Option<&dyn SeriesMetadataResolver>,
    force: bool,
) -> Result<usize, Box<dyn Error>> {
    let mut indexed = 0;

    for target in config
        .targets
        .iter()
        .filter(|target| target.target_type == TargetType::Series)
    {
        match index_target(target, repository, anime_resolver, force) {
            Ok(target_indexed) => indexed += target_indexed,
            Err(error) => warn!("failed indexing Jellyfin target {:?}: {error}", target.name),
        }
    }

    info!("finished Jellyfin series index: {indexed} indexed series");
    Ok(indexed)
}

fn index_target(
    target: &TargetConfig,
    repository: &SeriesMetadataRepository,
    anime_resolver: Option<&dyn SeriesMetadataResolver>,
    force: bool,
) -> Result<usize, Box<dyn Error>> {
    target.validate_source()?;
    let series_config = target.series.as_ref().ok_or_else(|| {
        format!(
            "Jellyfin target {:?} has no series configuration",
            target.name
        )
    })?;
    info!("indexing Jellyfin series in {}", target.source.display());
    let mut indexed = 0;

    for entry in fs::read_dir(&target.source)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warn!(
                    "failed reading an entry in Jellyfin target {:?}: {error}",
                    target.name
                );
                continue;
            }
        };
        let directory = entry.path();
        if !directory.is_dir() || directory.is_symlink() || target.ignores(&directory) {
            continue;
        }

        match index_series_directory(
            &directory,
            &series_config.category,
            repository,
            anime_resolver,
            force,
        ) {
            Ok(true) => {
                indexed += 1;
                info!("indexed Jellyfin series: {}", directory.display());
            }
            Ok(false) => {}
            Err(error) => warn!(
                "failed indexing Jellyfin series {} in target {:?}: {error}",
                directory.display(),
                target.name
            ),
        }
    }

    Ok(indexed)
}

fn index_series_directory(
    directory: &Path,
    category: &SeriesCategory,
    repository: &SeriesMetadataRepository,
    anime_resolver: Option<&dyn SeriesMetadataResolver>,
    force: bool,
) -> Result<bool, Box<dyn Error>> {
    find_indexed_or_index_series_directory(directory, category, repository, anime_resolver, force)
        .map(|(_, indexed)| indexed)
}

fn find_indexed_or_index_series_directory(
    directory: &Path,
    category: &SeriesCategory,
    repository: &SeriesMetadataRepository,
    anime_resolver: Option<&dyn SeriesMetadataResolver>,
    force: bool,
) -> Result<(SeriesMetadata, bool), Box<dyn Error>> {
    let directory = fs::canonicalize(directory)?;
    let path = directory
        .to_str()
        .ok_or_else(|| format!("series path {} is not valid UTF-8", directory.display()))?;
    let indexed_metadata = repository.find_by_path(path)?;
    if let Some(metadata) = indexed_metadata.as_ref().filter(|_| !force) {
        debug!("series already indexed: {}", directory.display());
        return Ok((metadata.clone(), false));
    }

    let mut metadata = match category {
        SeriesCategory::Anime => {
            let resolver = anime_resolver
                .ok_or("myanimelist.api_key is required to index an anime Jellyfin target")?;
            let resolved = resolver.resolve(&directory, category)?;
            SeriesMetadata::new(resolved.title, path.to_string(), Some(resolved.season))?
        }
        _ => SeriesMetadata::from_directory(&directory, None)?,
    };
    if let Some(indexed_metadata) = indexed_metadata {
        metadata.id = indexed_metadata.id;
    }
    repository.save(&metadata)?;

    Ok((metadata, true))
}

struct IndexedSeriesMetadataResolver<'a> {
    repository: &'a SeriesMetadataRepository,
    anime_resolver: Option<&'a dyn SeriesMetadataResolver>,
}

impl SeriesMetadataResolver for IndexedSeriesMetadataResolver<'_> {
    fn resolve(
        &self,
        source_directory: &Path,
        category: &SeriesCategory,
    ) -> Result<ResolvedSeriesMetadata, Box<dyn Error>> {
        let (metadata, indexed) = find_indexed_or_index_series_directory(
            source_directory,
            category,
            self.repository,
            self.anime_resolver,
            false,
        )?;
        if indexed {
            info!(
                "indexed Jellyfin series while mounting: {}",
                source_directory.display()
            );
        }

        Ok(ResolvedSeriesMetadata {
            title: metadata.title,
            season: metadata.season,
        })
    }
}

fn mount(
    config: &JellyfinConfig,
    repository: &SeriesMetadataRepository,
    anime_resolver: Option<&dyn SeriesMetadataResolver>,
) -> Result<usize, Box<dyn Error>> {
    let mut links = 0;

    for target in &config.targets {
        let result = match &target.target_type {
            TargetType::Series => {
                let resolver = IndexedSeriesMetadataResolver {
                    repository,
                    anime_resolver,
                };
                series::organize(target, &resolver)
            }
            TargetType::Movies => movies::organize(target),
        };

        match result {
            Ok(target_links) => links += target_links,
            Err(error) => warn!(
                "failed mounting Jellyfin target {:?} at {}: {error}",
                target.name,
                target.destination.display()
            ),
        }
    }

    info!("finished mounting Jellyfin structure: {links} symbolic links");
    Ok(links)
}

fn anime_resolver(config: &JellyfinConfig) -> Result<Option<impl SeriesMetadataResolver>, String> {
    let has_anime_target = config.targets.iter().any(|target| {
        target.target_type == TargetType::Series
            && target
                .series
                .as_ref()
                .is_some_and(|series| series.category == SeriesCategory::Anime)
    });
    if !has_anime_target {
        return Ok(None);
    }

    let resolver = AnimeResolver::build().map_err(|error| error.to_string())?;
    Ok(Some(resolver))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::tests::test_db_connection;
    use crate::jellyfin::config::{EpisodeConfig, EpisodeFallback, SeriesCategory, SeriesConfig};
    use serial_test::serial;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn indexes_discovered_series_once_and_defaults_season_to_one() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        let series_directory = source.join("Example Series");
        let second_series_directory = source.join("Second Series");
        let ignored_directory = source.join("Ignored Series");
        fs::create_dir_all(&series_directory).unwrap();
        fs::create_dir(&second_series_directory).unwrap();
        fs::create_dir(&ignored_directory).unwrap();

        let target = TargetConfig {
            name: "series".into(),
            target_type: TargetType::Series,
            source,
            destination,
            series: Some(SeriesConfig {
                category: SeriesCategory::Animation,
                episode: EpisodeConfig {
                    patterns: Vec::new(),
                    fallback: EpisodeFallback::FilesystemOrder,
                    start_at: 1,
                },
            }),
            include: vec!["*.mkv".into()],
            ignore: vec!["Ignored*".into()],
        };
        let config = JellyfinConfig {
            targets: vec![target],
        };
        let repository = SeriesMetadataRepository::new(test_db_connection());

        assert_eq!(index(&config, &repository, None, false).unwrap(), 2);
        assert_eq!(index(&config, &repository, None, false).unwrap(), 0);

        let canonical_path = fs::canonicalize(&series_directory).unwrap();
        let metadata = repository
            .find_by_path(canonical_path.to_str().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(metadata.title, "Example Series");
        assert_eq!(metadata.season, 1);
        let second_path = fs::canonicalize(second_series_directory).unwrap();
        assert!(
            repository
                .find_by_path(second_path.to_str().unwrap())
                .unwrap()
                .is_some()
        );
        let ignored_path = fs::canonicalize(ignored_directory).unwrap();
        assert!(
            repository
                .find_by_path(ignored_path.to_str().unwrap())
                .unwrap()
                .is_none()
        );
    }

    struct FixedAnimeResolver;

    impl SeriesMetadataResolver for FixedAnimeResolver {
        fn resolve(
            &self,
            _source_directory: &Path,
            category: &SeriesCategory,
        ) -> Result<ResolvedSeriesMetadata, Box<dyn Error>> {
            assert_eq!(category, &SeriesCategory::Anime);
            Ok(ResolvedSeriesMetadata {
                title: "Canonical Anime Title".into(),
                season: 3,
            })
        }
    }

    struct SelectiveAnimeResolver;

    impl SeriesMetadataResolver for SelectiveAnimeResolver {
        fn resolve(
            &self,
            source_directory: &Path,
            category: &SeriesCategory,
        ) -> Result<ResolvedSeriesMetadata, Box<dyn Error>> {
            assert_eq!(category, &SeriesCategory::Anime);
            if source_directory.ends_with("Broken Anime") {
                return Err("simulated resolver failure".into());
            }

            Ok(ResolvedSeriesMetadata {
                title: "Working Anime".into(),
                season: 2,
            })
        }
    }

    struct UnexpectedAnimeResolver;

    impl SeriesMetadataResolver for UnexpectedAnimeResolver {
        fn resolve(
            &self,
            _source_directory: &Path,
            _category: &SeriesCategory,
        ) -> Result<ResolvedSeriesMetadata, Box<dyn Error>> {
            panic!("the indexed metadata should have been reused")
        }
    }

    struct UpdatedAnimeResolver;

    impl SeriesMetadataResolver for UpdatedAnimeResolver {
        fn resolve(
            &self,
            _source_directory: &Path,
            category: &SeriesCategory,
        ) -> Result<ResolvedSeriesMetadata, Box<dyn Error>> {
            assert_eq!(category, &SeriesCategory::Anime);
            Ok(ResolvedSeriesMetadata {
                title: "Updated Anime Title".into(),
                season: 4,
            })
        }
    }

    #[test]
    #[serial]
    fn resolves_anime_title_and_season_before_indexing() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        let series_directory = source.join("Downloaded Anime Title");
        fs::create_dir_all(&series_directory).unwrap();

        let target = TargetConfig {
            name: "anime".into(),
            target_type: TargetType::Series,
            source,
            destination,
            series: Some(SeriesConfig {
                category: SeriesCategory::Anime,
                episode: EpisodeConfig {
                    patterns: Vec::new(),
                    fallback: EpisodeFallback::FilesystemOrder,
                    start_at: 1,
                },
            }),
            include: vec!["*.mkv".into()],
            ignore: Vec::new(),
        };
        let config = JellyfinConfig {
            targets: vec![target],
        };
        let repository = SeriesMetadataRepository::new(test_db_connection());

        assert_eq!(
            index(&config, &repository, Some(&FixedAnimeResolver), false,).unwrap(),
            1
        );

        let canonical_path = fs::canonicalize(series_directory).unwrap();
        let metadata = repository
            .find_by_path(canonical_path.to_str().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(metadata.title, "Canonical Anime Title");
        assert_eq!(metadata.season, 3);
    }

    #[test]
    #[serial]
    fn force_reindexes_and_updates_an_existing_series() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let series_directory = source.join("Downloaded Anime Title");
        fs::create_dir_all(&series_directory).unwrap();

        let config = JellyfinConfig {
            targets: vec![TargetConfig {
                name: "anime".into(),
                target_type: TargetType::Series,
                source,
                destination: temporary.path().join("destination"),
                series: Some(SeriesConfig {
                    category: SeriesCategory::Anime,
                    episode: EpisodeConfig {
                        patterns: Vec::new(),
                        fallback: EpisodeFallback::FilesystemOrder,
                        start_at: 1,
                    },
                }),
                include: vec!["*.mkv".into()],
                ignore: Vec::new(),
            }],
        };
        let repository = SeriesMetadataRepository::new(test_db_connection());
        let canonical_path = fs::canonicalize(&series_directory).unwrap();
        let path = canonical_path.to_str().unwrap();
        let original = SeriesMetadata::new("Old Anime Title".into(), path.into(), Some(1)).unwrap();
        repository.save(&original).unwrap();

        assert_eq!(
            index(&config, &repository, Some(&UnexpectedAnimeResolver), false,).unwrap(),
            0
        );
        assert_eq!(
            index(&config, &repository, Some(&UpdatedAnimeResolver), true,).unwrap(),
            1
        );

        let updated = repository.find_by_path(path).unwrap().unwrap();
        assert_eq!(updated.id, original.id);
        assert_eq!(updated.title, "Updated Anime Title");
        assert_eq!(updated.season, 4);
    }

    #[test]
    #[serial]
    fn continues_with_the_next_series_when_one_fails() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        let broken_directory = source.join("Broken Anime");
        let working_directory = source.join("Working Anime");
        fs::create_dir_all(&broken_directory).unwrap();
        fs::create_dir(&working_directory).unwrap();

        let target = TargetConfig {
            name: "anime".into(),
            target_type: TargetType::Series,
            source,
            destination,
            series: Some(SeriesConfig {
                category: SeriesCategory::Anime,
                episode: EpisodeConfig {
                    patterns: Vec::new(),
                    fallback: EpisodeFallback::FilesystemOrder,
                    start_at: 1,
                },
            }),
            include: vec!["*.mkv".into()],
            ignore: Vec::new(),
        };
        let config = JellyfinConfig {
            targets: vec![target],
        };
        let repository = SeriesMetadataRepository::new(test_db_connection());

        assert_eq!(
            index(&config, &repository, Some(&SelectiveAnimeResolver), false,).unwrap(),
            1
        );

        let broken_path = fs::canonicalize(broken_directory).unwrap();
        assert!(
            repository
                .find_by_path(broken_path.to_str().unwrap())
                .unwrap()
                .is_none()
        );
        let working_path = fs::canonicalize(working_directory).unwrap();
        let metadata = repository
            .find_by_path(working_path.to_str().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(metadata.title, "Working Anime");
        assert_eq!(metadata.season, 2);
    }

    #[test]
    #[serial]
    fn continues_with_the_next_target_when_one_fails() {
        let temporary = tempdir().unwrap();
        let missing_source = temporary.path().join("missing");
        let valid_source = temporary.path().join("valid");
        let series_directory = valid_source.join("Example Series");
        fs::create_dir_all(&series_directory).unwrap();

        let series_config = || {
            Some(SeriesConfig {
                category: SeriesCategory::Animation,
                episode: EpisodeConfig {
                    patterns: Vec::new(),
                    fallback: EpisodeFallback::FilesystemOrder,
                    start_at: 1,
                },
            })
        };
        let config = JellyfinConfig {
            targets: vec![
                TargetConfig {
                    name: "broken".into(),
                    target_type: TargetType::Series,
                    source: missing_source,
                    destination: temporary.path().join("broken-destination"),
                    series: series_config(),
                    include: vec!["*.mkv".into()],
                    ignore: Vec::new(),
                },
                TargetConfig {
                    name: "working".into(),
                    target_type: TargetType::Series,
                    source: valid_source,
                    destination: temporary.path().join("working-destination"),
                    series: series_config(),
                    include: vec!["*.mkv".into()],
                    ignore: Vec::new(),
                },
            ],
        };
        let repository = SeriesMetadataRepository::new(test_db_connection());

        assert_eq!(index(&config, &repository, None, false).unwrap(), 1);

        let path = fs::canonicalize(series_directory).unwrap();
        assert!(
            repository
                .find_by_path(path.to_str().unwrap())
                .unwrap()
                .is_some()
        );
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn mounts_a_series_using_the_index_and_indexes_it_when_missing() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        let series_directory = source.join("Downloaded Anime Title");
        fs::create_dir_all(&series_directory).unwrap();
        let episode = series_directory.join("episode.mkv");
        fs::write(&episode, b"episode").unwrap();

        let config = JellyfinConfig {
            targets: vec![TargetConfig {
                name: "anime".into(),
                target_type: TargetType::Series,
                source,
                destination: destination.clone(),
                series: Some(SeriesConfig {
                    category: SeriesCategory::Anime,
                    episode: EpisodeConfig {
                        patterns: Vec::new(),
                        fallback: EpisodeFallback::FilesystemOrder,
                        start_at: 1,
                    },
                }),
                include: vec!["*.mkv".into()],
                ignore: Vec::new(),
            }],
        };
        let repository = SeriesMetadataRepository::new(test_db_connection());

        assert_eq!(
            mount(&config, &repository, Some(&FixedAnimeResolver)).unwrap(),
            1
        );
        let link = destination
            .join("Canonical Anime Title")
            .join("Season 03")
            .join("Canonical Anime Title - S03E01.mkv");
        assert_eq!(
            fs::read_link(&link).unwrap(),
            fs::canonicalize(&episode).unwrap()
        );

        let indexed_path = fs::canonicalize(series_directory).unwrap();
        let metadata = repository
            .find_by_path(indexed_path.to_str().unwrap())
            .unwrap()
            .unwrap();
        assert_eq!(metadata.title, "Canonical Anime Title");
        assert_eq!(metadata.season, 3);

        assert_eq!(
            mount(&config, &repository, Some(&UnexpectedAnimeResolver)).unwrap(),
            1
        );
    }
}
