use crate::fs::{sanitize_filename_component, symlink_file};
use crate::jellyfin::config::{SeriesCategory, TargetConfig, TargetType};
use crate::jellyfin::pattern::EpisodePatterns;
use log::warn;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSeriesMetadata {
    pub title: String,
    pub season: u32,
}

pub trait SeriesMetadataResolver {
    fn resolve(
        &self,
        source_directory: &Path,
        category: &SeriesCategory,
    ) -> Result<ResolvedSeriesMetadata, Box<dyn Error>>;
}

pub fn organize(
    target: &TargetConfig,
    metadata_resolver: &impl SeriesMetadataResolver,
) -> Result<usize, Box<dyn Error>> {
    if target.target_type != TargetType::Series {
        return Err(format!("Jellyfin target {:?} is not a series target", target.name).into());
    }
    target.validate()?;
    target.validate_source()?;
    fs::create_dir_all(&target.destination)?;

    let series_config = target.series.as_ref().ok_or_else(|| {
        format!(
            "Jellyfin target {:?} has no series configuration",
            target.name
        )
    })?;
    let patterns = EpisodePatterns::compile(&series_config.episode)?;
    let mut links = 0;

    for entry in fs::read_dir(&target.source)? {
        let entry = entry?;
        let series_directory = entry.path();
        if !series_directory.is_dir()
            || series_directory.is_symlink()
            || target.ignores(&series_directory)
        {
            continue;
        }

        let files = media_files(target, &series_directory)?;
        if files.is_empty() {
            continue;
        }

        let metadata = match metadata_resolver.resolve(&series_directory, &series_config.category) {
            Ok(metadata) => metadata,
            Err(error) => {
                warn!(
                    "failed resolving metadata for series {}: {error}",
                    series_directory.display()
                );
                continue;
            }
        };
        if let Err(error) = validate_metadata(&metadata) {
            warn!(
                "invalid metadata for series {}: {error}",
                series_directory.display()
            );
            continue;
        }
        let episodes = match patterns.number_files(&files) {
            Ok(episodes) => episodes,
            Err(error) => {
                warn!(
                    "failed numbering episodes for series {}: {error}",
                    series_directory.display()
                );
                continue;
            }
        };
        let safe_title = sanitize_filename_component(&metadata.title);
        let season_directory = target
            .destination
            .join(&safe_title)
            .join(format!("Season {:02}", metadata.season));
        fs::create_dir_all(&season_directory)?;

        for episode in episodes {
            let Some(extension) = episode.path.extension() else {
                warn!(
                    "episode file {} has no extension; skipping it",
                    episode.path.display()
                );
                continue;
            };
            let extension = extension.to_string_lossy();
            let file_name = sanitize_filename_component(&format!(
                "{} - S{:02}E{:02}.{}",
                safe_title, metadata.season, episode.number, extension
            ));
            let destination = season_directory.join(file_name);
            let source = fs::canonicalize(&episode.path)?;
            symlink_file(&source, &destination)?;
            links += 1;
        }
    }

    Ok(links)
}

fn media_files(target: &TargetConfig, directory: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !target.ignores(entry.path()))
    {
        let entry = entry?;
        if entry.file_type().is_file() && target.includes(entry.path()) {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}

fn validate_metadata(metadata: &ResolvedSeriesMetadata) -> Result<(), Box<dyn Error>> {
    if metadata.title.trim().is_empty() {
        return Err("resolved series title cannot be empty".into());
    }
    if metadata.season == 0 {
        return Err("resolved series season must be greater than zero".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jellyfin::config::{EpisodeConfig, EpisodeFallback, SeriesConfig, TargetConfig};
    use tempfile::tempdir;

    struct FixedResolver;

    impl SeriesMetadataResolver for FixedResolver {
        fn resolve(
            &self,
            _source_directory: &Path,
            _category: &SeriesCategory,
        ) -> Result<ResolvedSeriesMetadata, Box<dyn Error>> {
            Ok(ResolvedSeriesMetadata {
                title: "Example: Anime?/Part".into(),
                season: 2,
            })
        }
    }

    struct ConditionalResolver;

    impl SeriesMetadataResolver for ConditionalResolver {
        fn resolve(
            &self,
            source_directory: &Path,
            _category: &SeriesCategory,
        ) -> Result<ResolvedSeriesMetadata, Box<dyn Error>> {
            if source_directory.ends_with("Broken") {
                return Err("metadata resolution failed".into());
            }

            Ok(ResolvedSeriesMetadata {
                title: "Working".into(),
                season: 1,
            })
        }
    }

    #[test]
    fn validates_resolved_metadata() {
        assert!(
            validate_metadata(&ResolvedSeriesMetadata {
                title: String::new(),
                season: 1,
            })
            .is_err()
        );
        assert!(
            validate_metadata(&ResolvedSeriesMetadata {
                title: "Example".into(),
                season: 0,
            })
            .is_err()
        );
        assert!(
            validate_metadata(
                &FixedResolver
                    .resolve(Path::new("Example"), &SeriesCategory::Anime,)
                    .unwrap()
            )
            .is_ok()
        );
    }

    #[cfg(unix)]
    #[test]
    fn organizes_a_series_using_resolved_metadata_and_filesystem_order() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let series_source = source.join("Downloaded name");
        let destination = temporary.path().join("destination");
        fs::create_dir_all(&series_source).unwrap();
        let episode_two = series_source.join("show 2.mkv");
        let episode_ten = series_source.join("show 10.mkv");
        fs::write(&episode_ten, b"ten").unwrap();
        fs::write(&episode_two, b"two").unwrap();

        let target = TargetConfig {
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
        };

        let filesystem_order = media_files(&target, &series_source).unwrap();
        assert_eq!(organize(&target, &FixedResolver).unwrap(), 2);
        let season = destination.join("Example_ Anime_Part").join("Season 02");
        assert_eq!(
            fs::read_link(season.join("Example_ Anime_Part - S02E01.mkv")).unwrap(),
            fs::canonicalize(&filesystem_order[0]).unwrap()
        );
        assert_eq!(
            fs::read_link(season.join("Example_ Anime_Part - S02E02.mkv")).unwrap(),
            fs::canonicalize(&filesystem_order[1]).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn continues_with_the_next_series_when_one_fails() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let broken_series = source.join("Broken");
        let working_series = source.join("Working");
        let destination = temporary.path().join("destination");
        fs::create_dir_all(&broken_series).unwrap();
        fs::create_dir_all(&working_series).unwrap();
        fs::write(broken_series.join("episode.mkv"), b"broken").unwrap();
        let working_episode = working_series.join("episode.mkv");
        fs::write(&working_episode, b"working").unwrap();

        let target = TargetConfig {
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
        };

        assert_eq!(organize(&target, &ConditionalResolver).unwrap(), 1);
        assert_eq!(
            fs::read_link(
                destination
                    .join("Working")
                    .join("Season 01")
                    .join("Working - S01E01.mkv")
            )
            .unwrap(),
            fs::canonicalize(working_episode).unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn stops_when_a_series_filesystem_operation_fails() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let series_source = source.join("Series");
        let destination = temporary.path().join("destination");
        fs::create_dir_all(&series_source).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(series_source.join("episode.mkv"), b"episode").unwrap();

        // This file prevents creation of the resolved series directory.
        fs::write(destination.join("Example_ Anime_Part"), b"blocking file").unwrap();

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

        assert!(organize(&target, &FixedResolver).is_err());
    }
}
