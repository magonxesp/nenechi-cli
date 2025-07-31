use std::fmt::format;
use std::path::Path;

use crate::config::{TidyWallpapersConfig, WallpapersConfig};
use crate::fs::path_match_any_pattern;
use clap::Subcommand;
use glob::glob;
use log::{debug, info};
use serde::Deserialize;
use uuid::{NoContext, Timestamp, Uuid};
use walkdir::{DirEntry, WalkDir};
use nenechi_image::ImageDetails;
use nenechi_pixiv::{fetch_tags, IllustrationId};
use crate::ApplicationContext;
use crate::models::{Wallpaper, WallpaperRepository};

#[derive(Debug, Subcommand)]
pub enum WallpapersCommand {
    Tidy,
    Index,
    CleanIndex
}

pub fn execute_wallpapers_command(
    command: WallpapersCommand,
    context: &ApplicationContext
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        WallpapersCommand::Tidy => tidy_wallpapers(&context.config.wallpapers),
        WallpapersCommand::Index => Err("Not implemented".into()),
        WallpapersCommand::CleanIndex => Err("Not implemented".into())
    }
}

fn tidy_wallpapers(config: &WallpapersConfig) -> Result<(), Box<dyn std::error::Error>> {
    debug!("using config: {:?}", config);

    let directory = config.directory()?;
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

fn index_wallpapers(context: &ApplicationContext) -> Result<(), Box<dyn std::error::Error>> {
    let config = &context.config.wallpapers;
    let wallpapers_repository = WallpaperRepository::new(&mut context.db_connection);
    debug!("using config: {:?}", config);

    for file in walk_wallpapers_directory(config)? {
        let file = file?;
        let path = file.path();
        debug!("indexing wallpaper: {}", path.display());

        let id = Uuid::new_v7(Timestamp::now(NoContext))
            .to_string();
        let illustration_id = IllustrationId::from_path(path).ok();
        let mut tags: Vec<String> = vec![];

        if let Some(illustration_id) = illustration_id.clone() {
            tags = fetch_tags(&illustration_id)?
                .into_iter()
                .map(|tag| tag.translation.en)
                .collect();
        }

        let image_details = ImageDetails::read_from_path(path)?;
        let path_string = path.to_str()
            .ok_or(format!("unable to cast to string path {}", path.display()))?
            .to_string();
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

        // TODO: save wallpaper to sqlite
    }

    Ok(())
}

fn walk_wallpapers_directory(config: &WallpapersConfig) -> Result<impl Iterator<Item = Result<DirEntry, walkdir::Error>>, Box<dyn std::error::Error>> {
    let patterns = config.ignore.clone();
    let entry_filter = move |entry: &DirEntry| {
        if !entry.file_type().is_file() || entry.file_type().is_symlink() {
            debug!("path excluded because is not a file or is a symlink: {}", entry.path().display());
            return false;
        }

        if !patterns.is_empty() && path_match_any_pattern(entry.path(), &patterns) {
            debug!("path excluded because is ignored: {}", entry.path().display());
            return false;
        }

        true
    };

    let walker = WalkDir::new(config.directory()?)
        .into_iter()
        .filter_entry(entry_filter);

    Ok(walker)
}
