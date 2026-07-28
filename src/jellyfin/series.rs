use crate::fs::symlink_file;
use crate::jellyfin::config::{SeriesCategory, TargetConfig, TargetType};
use crate::jellyfin::pattern::EpisodePatterns;
use std::error::Error;
use std::fs;
use std::path::{Component, Path, PathBuf};
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

        let metadata = metadata_resolver.resolve(&series_directory, &series_config.category)?;
        validate_metadata(&metadata)?;
        let episodes = patterns.number_files(&files)?;
        let season_directory = target
            .destination
            .join(&metadata.title)
            .join(format!("Season {:02}", metadata.season));
        fs::create_dir_all(&season_directory)?;

        for episode in episodes {
            let extension = episode
                .path
                .extension()
                .ok_or_else(|| format!("episode file {} has no extension", episode.path.display()))?
                .to_string_lossy();
            let destination = season_directory.join(format!(
                "{} - S{:02}E{:02}.{}",
                metadata.title, metadata.season, episode.number, extension
            ));
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

    let path = Path::new(&metadata.title);
    if path.components().count() != 1
        || !matches!(path.components().next(), Some(Component::Normal(_)))
    {
        return Err(format!(
            "resolved series title {:?} is not a safe directory name",
            metadata.title
        )
        .into());
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
                title: "Example".into(),
                season: 2,
            })
        }
    }

    #[test]
    fn validates_resolved_metadata() {
        assert!(
            validate_metadata(&ResolvedSeriesMetadata {
                title: "../unsafe".into(),
                season: 1,
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
        let season = destination.join("Example").join("Season 02");
        assert_eq!(
            fs::read_link(season.join("Example - S02E01.mkv")).unwrap(),
            fs::canonicalize(&filesystem_order[0]).unwrap()
        );
        assert_eq!(
            fs::read_link(season.join("Example - S02E02.mkv")).unwrap(),
            fs::canonicalize(&filesystem_order[1]).unwrap()
        );
    }
}
