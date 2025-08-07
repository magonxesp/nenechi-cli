use std::path::Path;
use crate::config::WallpapersConfig;
use crate::fs::path_match_any_pattern;
use crate::models::{Wallpaper, WallpaperRepository};
use clap::Subcommand;
use log::{debug, info, warn};
use nenechi_image::{is_image_file, ImageDetails};
use nenechi_pixiv::{fetch_tags, IllustrationId};
use std::thread::sleep;
use std::time::Duration;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use uuid::{NoContext, Timestamp, Uuid};
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Subcommand)]
pub enum WallpapersCommands {
    Tidy,
    Index,
    CleanIndex
}

pub fn execute_wallpaper_command(
    command: WallpapersCommands,
    config: WallpapersConfig,
    connection_pool: Pool<ConnectionManager<SqliteConnection>>
) -> Result<(), Box<dyn std::error::Error>> {
    let ignore_patterns = config.ignore.clone();
    let directory = config.directory()?;
    let wallpapers_repository = WallpaperRepository::new(connection_pool);

    match command {
        WallpapersCommands::Tidy => tidy_command(directory),
        WallpapersCommands::Index => index(
            ignore_patterns,
            directory,
            &wallpapers_repository
        ),
        WallpapersCommands::CleanIndex => Err("Not implemented".into())
    }
}

fn tidy_command(directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    info!("tidying wallpapers for directory: {}", directory.display());

    for entry in WalkDir::new(directory) {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_file() && !metadata.is_symlink() {
            info!("Archivo: {}", path.display());
        }
    }

    Ok(())
}

fn index(
    ignore_patterns: Vec<String>,
    directory: &Path,
    wallpapers_repository: &WallpaperRepository
) -> Result<(), Box<dyn std::error::Error>> {
    info!("indexing wallpapers for directory: {}", directory.display());

    let walker = walk_directory(ignore_patterns, directory)?;

    for file in walker {
        let file = file?;
        let path = file.path();
        if !file.file_type().is_file() || !is_image_file(file.path()) {
            debug!("file is not an image, skipping: {}", path.display());
            continue
        }

        let index_result = index_file(&file, wallpapers_repository);
        if let Err(e) = index_result {
            warn!("wallpaper index failed for {}: {}", path.display(), e)
        }
    }

    info!("finish wallpapers index for directory: {}", directory.display());
    Ok(())
}

/// index the file if it is not indexed
fn index_file(
    file: &DirEntry,
    wallpapers_repository: &WallpaperRepository
) -> Result<(), Box<dyn std::error::Error>> {
    let path = file.path();
    let path_string = path.to_str()
        .ok_or(format!("unable to cast to string path {}", path.display()))?
        .to_string();
    debug!("indexing wallpaper: {}", &path_string);

    let existing = wallpapers_repository.find_by_path(&path_string)?;
    if existing.is_some() {
        debug!("wallpaper already indexed: {}", &path_string);
        return Ok(());
    }

    let id = Uuid::new_v7(Timestamp::now(NoContext)).to_string();
    let illustration_id = IllustrationId::from_path(path).ok();
    let tags = resolve_image_tags(path);
    let image_details = ImageDetails::read_from_path(path)?;
    let file_name = path.file_name()
        .ok_or(format!("unable to get file name for {}", path.display()))?
        .to_str()
        .ok_or(format!("unable to cast file name to string for {}", path.display()))?
        .to_string();

    let wallpaper = Wallpaper {
        id,
        pixiv_illustration_id: illustration_id.map(|id| id.value),
        tags,
        aspect_ratio: image_details.aspect_ratio,
        path: path_string,
        file_name,
    };

    wallpapers_repository.save(&wallpaper)?;
    debug!("wallpaper indexed: {}", &wallpaper.path);
    Ok(())
}

fn resolve_image_tags(path: &Path) -> Vec<String> {
    let illustration_id = match IllustrationId::from_path(path) {
        Ok(id) => id,
        Err(e) => {
            debug!("error resolving Pixiv illustration id for path {}: {}; ", path.display(), e);
            return vec![]
        }
    };

    let tags = match fetch_tags(&illustration_id) {
        Ok(tags) => tags,
        Err(e) => {
            warn!("failed fetching Pixiv tags for path {}: {}", path.display(), e);
            return vec![]
        }
    };

    sleep(Duration::from_millis(300));
    tags.iter()
        .map(|tag| tag.translation.en.clone())
        .collect()
}

fn walk_directory(
    ignore_patterns: Vec<String>,
    directory: &Path
) -> Result<impl Iterator<Item = Result<DirEntry, walkdir::Error>> + use<>, Box<dyn std::error::Error>> {
    debug!("walking directory: {}", directory.display());

    let entry_filter = move |entry: &DirEntry| {
        if entry.file_type().is_symlink() {
            debug!("path excluded because is a symlink: {}", entry.path().display());
            return false;
        }

        if !ignore_patterns.is_empty() && path_match_any_pattern(entry.path(), &ignore_patterns) {
            debug!("path excluded because is ignored: {}", entry.path().display());
            return false;
        }

        true
    };

    let walker = WalkDir::new(directory)
        .into_iter()
        .filter_entry(entry_filter);

    Ok(walker)
}


