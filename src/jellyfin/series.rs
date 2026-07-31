use crate::fs::{sanitize_filename_component, symlink_file};
use crate::jellyfin::config::{SeriesCategory, SeriesConfig, TargetConfig, TargetType};
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
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warn!(
                    "failed reading a series item in {}: {error}",
                    target.source.display()
                );
                continue;
            }
        };
        let series_directory = entry.path();
        if !series_directory.is_dir()
            || series_directory.is_symlink()
            || target.ignores(&series_directory)
        {
            continue;
        }

        match organize_series_directory(
            target,
            &series_directory,
            series_config,
            &patterns,
            metadata_resolver,
        ) {
            Ok(directory_links) => links += directory_links,
            Err(error) => warn!(
                "failed organizing series directory {}: {error}",
                series_directory.display()
            ),
        }
    }

    Ok(links)
}

fn organize_series_directory(
    target: &TargetConfig,
    series_directory: &Path,
    series_config: &SeriesConfig,
    patterns: &EpisodePatterns,
    metadata_resolver: &impl SeriesMetadataResolver,
) -> Result<usize, Box<dyn Error>> {
    let files = media_files(target, series_directory);
    if files.is_empty() {
        return Ok(0);
    }

    let metadata = metadata_resolver.resolve(series_directory, &series_config.category)?;
    validate_metadata(&metadata)?;
    let episodes = patterns.number_files(&files)?;
    let safe_title = sanitize_filename_component(&metadata.title);
    let season_directory = target
        .destination
        .join(&safe_title)
        .join(format!("Season {:02}", metadata.season));
    fs::create_dir_all(&season_directory)?;

    let mut links = 0;
    for episode in episodes {
        let result = (|| {
            let extension = episode
                .path
                .extension()
                .ok_or_else(|| format!("episode file {} has no extension", episode.path.display()))?
                .to_string_lossy();
            let file_name = sanitize_filename_component(&format!(
                "{} - S{:02}E{:02}.{}",
                safe_title, metadata.season, episode.number, extension
            ));
            let destination = season_directory.join(file_name);
            let source = fs::canonicalize(&episode.path)?;
            symlink_file(&source, &destination)
        })();
        match result {
            Ok(()) => links += 1,
            Err(error) => warn!(
                "failed organizing episode {}: {error}",
                episode.path.display()
            ),
        }
    }

    Ok(links)
}

fn media_files(target: &TargetConfig, directory: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !target.ignores(entry.path()))
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warn!(
                    "failed reading an item in series directory {}: {error}",
                    directory.display()
                );
                continue;
            }
        };
        if entry.file_type().is_file() && target.includes(entry.path()) {
            files.push(entry.into_path());
        }
    }
    files
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

        let filesystem_order = media_files(&target, &series_source);
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
}
