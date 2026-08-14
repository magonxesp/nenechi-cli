use crate::fs::symlink_file;
use crate::jellyfin::config::{TargetConfig, TargetType};
use crate::jellyfin::metadata::{SeriesNfo, write_poster};
use crate::jellyfin::pattern::EpisodePatterns;
use crate::media;
use crate::media::SeriesMetadata;
use log::{debug, warn};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn organize(target: &TargetConfig) -> Result<usize, String> {
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

        let metadata = media::read_series_metadata_from_path(&series_directory);
        if let Err(err) = &metadata {
            warn!("error reading metadata: {}", err);
            continue;
        }

        let metadata = metadata?;
        debug!("scanning {:?}", series_directory);
        let files = media_files(target, &series_directory);

        let destination_series_directory = target
            .destination
            .join(&metadata.title);
        let destination_season_directory = destination_series_directory
            .join(format!("Season {:02}", metadata.season));
        fs::create_dir_all(&destination_season_directory).map_err(|e| e.to_string())?;

        let nfo = SeriesNfo::from(metadata.clone());

        if metadata.season == 1 {
            nfo.write(&destination_series_directory, false)?;
        } else {
            nfo.write(&destination_season_directory, true)?;
        }

        if let Some(cover) = &metadata.cover {
            if metadata.season == 1 {
                write_poster(&destination_series_directory, &cover)?;
                write_poster(&destination_season_directory, &cover)?;
            } else {
                write_poster(&destination_season_directory, &cover)?;
            }
        }

        for file in files {
            let result = create_episode_symlink(
                &file,
                &destination_season_directory,
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
