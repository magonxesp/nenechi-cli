use std::path::Path;
use crate::config::WallpapersConfig;
use crate::fs::path_match_any_pattern;
use crate::models::{Wallpaper, WallpaperRepository};
use clap::Subcommand;
use log::{debug, info};
use nenechi_image::ImageDetails;
use nenechi_pixiv::{fetch_tags, IllustrationId};
use std::thread::sleep;
use std::time::Duration;
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
    db_connection: &mut SqliteConnection
) -> Result<(), Box<dyn std::error::Error>> {
    let ignore_patterns = config.ignore.clone();
    let directory = config.directory()?;
    let mut wallpapers_repository = WallpaperRepository::new(db_connection);

    match command {
        WallpapersCommands::Tidy => tidy_command(directory),
        WallpapersCommands::Index => index(
            ignore_patterns,
            directory,
            &mut wallpapers_repository
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
    wallpapers_repository: &mut WallpaperRepository
) -> Result<(), Box<dyn std::error::Error>> {
    for file in walk_directory(ignore_patterns, directory)? {
        let file = file?;
        index_file(file, wallpapers_repository)?
    }

    Ok(())
}

/// index the file if it is not indexed
fn index_file(
    file: DirEntry,
    wallpapers_repository: &mut WallpaperRepository
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

    let id = Uuid::new_v7(Timestamp::now(NoContext))
        .to_string();
    let illustration_id = IllustrationId::from_path(path).ok();
    let mut tags: Vec<String> = vec![];

    if let Some(illustration_id) = illustration_id.clone() {
        tags = fetch_tags(&illustration_id)?
            .into_iter()
            .map(|tag| tag.translation.en)
            .collect();
        sleep(Duration::from_millis(300));
    }

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

fn walk_directory(
    ignore_patterns: Vec<String>,
    directory: &Path
) -> Result<impl Iterator<Item = Result<DirEntry, walkdir::Error>> + use<>, Box<dyn std::error::Error>> {
    let entry_filter = move |entry: &DirEntry| {
        if !entry.file_type().is_file() || entry.file_type().is_symlink() {
            debug!("path excluded because is not a file or is a symlink: {}", entry.path().display());
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


