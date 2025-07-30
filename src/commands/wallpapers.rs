use std::path::Path;

use crate::config::{TidyWallpapersConfig, WallpapersConfig};
use crate::fs::path_match_any_pattern;
use clap::Subcommand;
use glob::glob;
use log::{debug, info};
use serde::Deserialize;
use walkdir::{DirEntry, WalkDir};

#[derive(Debug, Subcommand)]
pub enum WallpapersCommand {
    Tidy,
    Index,
    CleanIndex
}

pub fn execute_wallpapers_command(
    command: WallpapersCommand,
    config: &WallpapersConfig
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        WallpapersCommand::Tidy => tidy_wallpapers(config),
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

fn index_wallpapers(config: &WallpapersConfig) -> Result<(), Box<dyn std::error::Error>> {
    debug!("using config: {:?}", config);
    // TODO: recorrer ficheros con walk_wallpapers_directory y llamar al endpoint de pixiv para
    // recuperar los tags
    for file in walk_wallpapers_directory(config)? {

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
