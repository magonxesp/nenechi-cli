use crate::fs::symlink_file;
use crate::jellyfin::config::{SeriesCategory, SeriesConfig, TargetConfig, TargetType};
use crate::jellyfin::pattern::EpisodePatterns;
use log::{debug, warn};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;
use crate::jellyfin::series_metadata::{SeriesMetadata, SeriesMetadataRepository};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSeriesMetadata {
    // always the first season title
    pub title: String,
    pub season: i64,
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
    metadata_repository: &SeriesMetadataRepository,
) -> Result<usize, String> {
    if target.target_type != TargetType::Series {
        return Err(format!("Jellyfin target {:?} is not a series target", target.name));
    }

    target.validate()?;
    target.validate_source()?;
    fs::create_dir_all(&target.destination).map_err(|e| e.to_string())?;

    let series_config = target.series.as_ref().ok_or_else(|| {
        format!(
            "Jellyfin target {:?} has no series configuration",
            target.name
        )
    })?;
    let patterns = EpisodePatterns::compile(&series_config.episode)?;
    let mut links = 0;

    for entry in fs::read_dir(&target.source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let series_directory = entry.path();
        if !series_directory.is_dir()
            || series_directory.is_symlink()
            || target.ignores(&series_directory)
        {
            debug!("ignoring {:?}", series_directory);
            continue;
        }

        let metadata = match series_metadata(
            &series_directory,
            &series_config,
            &metadata_repository,
            metadata_resolver,
        ) {
            Ok(metadata) => metadata,
            Err(err) => {
                warn!("failed to get metadata for {:?}: {:?}", series_directory, err);
                continue;
            }
        };

        debug!("scanning {:?}", series_directory);
        let files = media_files(target, &series_directory);

        let season_directory = target
            .destination
            .join(&metadata.title)
            .join(format!("Season {:02}", metadata.season));
        fs::create_dir_all(&season_directory).map_err(|e| e.to_string())?;

        for file in files {
            let result = create_episode_symlink(
                &file,
                &season_directory,
                &metadata,
                &patterns
            );

            if let Err(err) = result {
                warn!("failed to create symlink for series {:?}: {err}", series_directory);
                continue;
            }

            links += 1;
        }
    }

    Ok(links)
}

fn create_episode_symlink(
    episode_path: &PathBuf,
    season_directory: &PathBuf,
    metadata: &SeriesMetadata,
    patterns: &EpisodePatterns
) -> Result<(), String> {
    let extension = episode_path.extension()
        .ok_or_else(|| format!("episode file {:?} has no extension", episode_path))?;

    let file_name = episode_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| format!("episode file {:?} has no filename", episode_path))?;

    let extension = extension.to_string_lossy();
    let episode_number = patterns.extract(file_name)?
        .ok_or_else(|| format!("episode file {:?} has no episode_number", episode_path))?;

    let file_name = format!(
        "{} - S{:02}E{:02}.{}",
        metadata.title, metadata.season, episode_number, extension
    );

    let destination = season_directory.join(file_name);
    let source = fs::canonicalize(&episode_path.as_path())
        .map_err(|e| format!("error canonicalizing file {:?}: {}", episode_path, e))?;
    symlink_file(&source, &destination)
        .map_err(|e| format!("error symlinking file {:?}: {}", episode_path, e))?;

    Ok(())
}

fn filter_media_entry(entry: &walkdir::DirEntry, target: &TargetConfig) -> bool {
    let path = entry.path();

    if !entry.file_type().is_file() {
        debug!("ignoring target {:?}: is not a file", path);
        return false;
    }

    if !target.includes(path) {
        debug!("ignoring target {:?}: is not wanted or included", path);
        return false;
    }

    debug!("found target {:?}", path);
    true
}

fn media_files(target: &TargetConfig, directory: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(directory)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !target.ignores(entry.path()))
        .filter_map(Result::ok)
        .filter(|entry| filter_media_entry(entry, target))
        .map(|entry| entry.into_path())
}

fn validate_metadata(metadata: &ResolvedSeriesMetadata) -> Result<(), String> {
    if metadata.title.trim().is_empty() {
        return Err("resolved series title cannot be empty".to_string());
    }
    if metadata.season == 0 {
        return Err("resolved series season must be greater than zero".to_string());
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

fn series_metadata(
    path: &Path,
    config: &SeriesConfig,
    repository: &SeriesMetadataRepository,
    resolver: &impl SeriesMetadataResolver
) -> Result<SeriesMetadata, String> {
    let path_str = path.to_string_lossy().to_string();
    let metadata = repository.find_by_path(path_str.as_str()).map_err(|e| e.to_string())?;
    if let Some(metadata) = metadata {
        return Ok(metadata);
    }

    let metadata = match resolver.resolve(&path, &config.category) {
        Ok(metadata) => metadata,
        Err(error) => return Err(format!("failed resolving metadata for series {:?}: {error}", path))
    };

    validate_metadata(&metadata)?;
    let metadata = SeriesMetadata::new(metadata.title, path_str, Some(metadata.season))?;
    repository.save(&metadata).map_err(|e| e.to_string())?;
    Ok(metadata)
}
