use crate::config::{TidyWallpapersConfig, WallpapersConfig};
use crate::fs::{path_match_any_pattern, symlink_file, unwrap_optional_os_str};
use crate::wallpapers::{Wallpaper, WallpaperRepository};
use clap::Subcommand;
use log::{debug, info, warn};
use nenechi_image::{ImageDetails, is_image_file};
use nenechi_pixiv::{IllustrationId, fetch_tags};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use std::{fmt, fs};
use uuid::{NoContext, Timestamp, Uuid};
use walkdir::{DirEntry, WalkDir};

#[derive(Clone, Debug, Subcommand)]
pub enum WallpapersCommands {
    Tidy,
    Index,
    CleanIndex,
}

impl Display for WallpapersCommands {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WallpapersCommands::Tidy => write!(f, "tidy"),
            WallpapersCommands::Index => write!(f, "index"),
            WallpapersCommands::CleanIndex => write!(f, "clean-index"),
        }
    }
}

pub fn execute_wallpaper_command(command: WallpapersCommands) -> Result<(), String> {
    let config = WallpapersConfig::read()?;
    let ignore_patterns = config.ignore.clone();
    let directory = config.directory()?;
    let result = match command {
        WallpapersCommands::Tidy => tidy(config.tidy()?, ignore_patterns, directory),
        WallpapersCommands::Index => index(ignore_patterns, directory),
        WallpapersCommands::CleanIndex => Err("Not implemented".into()),
    };

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(format!(
            "subcommand {} failed: {}",
            command.to_string(),
            err
        )),
    }
}

fn tidy(
    config: TidyWallpapersConfig,
    ignore_patterns: Vec<String>,
    directory: &Path,
) -> Result<(), Box<dyn Error>> {
    info!("tidying wallpapers for directory: {}", directory.display());
    let repository = WallpaperRepository::get_instance();
    let walker = walk_directory(ignore_patterns, directory)?;

    for file in walker {
        let file = file?;
        let path = file.path();

        if !file.file_type().is_file() || !is_image_file(file.path()) {
            debug!("file is not an image, skipping: {}", path.display());
            continue;
        }

        let wallpaper = find_indexed_or_index(&file, repository);

        if let Err(e) = wallpaper {
            warn!(
                "failed retrieving indexed wallpaper {}: {}",
                path.display(),
                e
            );
            continue;
        }

        create_wallpaper_symlinks(&config, path, &wallpaper.unwrap())?
    }

    info!(
        "finish tidying wallpapers for directory: {}",
        directory.display()
    );
    Ok(())
}

fn create_wallpaper_symlinks(
    config: &TidyWallpapersConfig,
    original: &Path,
    wallpaper: &Wallpaper,
) -> Result<(), Box<dyn Error>> {
    create_wallpaper_aspect_ratio_symlinks(config, original, wallpaper)?;
    create_wallpaper_tags_symlinks(config, original, wallpaper)?;

    Ok(())
}

fn create_wallpaper_aspect_ratio_symlinks(
    config: &TidyWallpapersConfig,
    original: &Path,
    wallpaper: &Wallpaper,
) -> Result<(), Box<dyn Error>> {
    let aspect_ratio_directory = config.aspect_ratio_directory()?;
    let aspect_ratio_directory = aspect_ratio_directory.join(wallpaper.aspect_ratio.to_string());

    if !aspect_ratio_directory.exists() {
        fs::create_dir_all(&aspect_ratio_directory)?;
    }

    let original_file_name = unwrap_optional_os_str(original.file_name())?;
    let destination_symlink = &aspect_ratio_directory.join(&original_file_name);
    let destination_symlink = Path::new(destination_symlink);
    symlink_file(original, destination_symlink).map_err(Box::from)
}

fn create_wallpaper_tags_symlinks(
    config: &TidyWallpapersConfig,
    original: &Path,
    wallpaper: &Wallpaper,
) -> Result<(), Box<dyn Error>> {
    let tags_directory = config.tags_directory()?;

    for tag in &wallpaper.tags {
        let tag = tag.trim();
        if tag.is_empty() {
            continue;
        }

        let tag_directory = tags_directory.join(tag);

        if !tag_directory.exists() {
            fs::create_dir_all(&tag_directory)?;
        }

        let original_file_name = unwrap_optional_os_str(original.file_name())?;
        let destination_symlink = &tag_directory.join(&original_file_name);
        let destination_symlink = Path::new(destination_symlink);
        symlink_file(original, destination_symlink)?
    }

    Ok(())
}

fn find_indexed_or_index(
    file: &DirEntry,
    repository: &WallpaperRepository,
) -> Result<Wallpaper, Box<dyn Error>> {
    let path = file.path();
    let path_string = path
        .to_str()
        .ok_or(format!("unable to cast to string path {}", path.display()))?
        .to_string();

    let existing = repository.find_by_path(&path_string)?;

    if existing.is_none() {
        info!(
            "wallpaper is not indexed, trying to index: {}",
            path.display()
        );
        let indexed = index_file(file, repository);
        if let Err(e) = indexed {
            warn!("wallpaper index failed for {}: {}", path.display(), e);
            return Err(e);
        }

        return Ok(indexed?);
    }

    Ok(existing.unwrap())
}

fn index(ignore_patterns: Vec<String>, directory: &Path) -> Result<(), Box<dyn Error>> {
    info!("indexing wallpapers for directory: {}", directory.display());
    let repository = WallpaperRepository::get_instance();
    let walker = walk_directory(ignore_patterns, directory)?;

    for file in walker {
        let file = file?;
        let path = file.path();
        if !file.file_type().is_file() || !is_image_file(file.path()) {
            debug!("file is not an image, skipping: {}", path.display());
            continue;
        }

        let index_result = index_file(&file, repository);
        if let Err(e) = index_result {
            warn!("wallpaper index failed for {}: {}", path.display(), e)
        }
    }

    info!(
        "finish wallpapers index for directory: {}",
        directory.display()
    );
    Ok(())
}

/// index the file if it is not indexed
fn index_file(
    file: &DirEntry,
    wallpapers_repository: &WallpaperRepository,
) -> Result<Wallpaper, Box<dyn Error>> {
    let path = file.path();
    let path_string = path
        .to_str()
        .ok_or(format!("unable to cast to string path {}", path.display()))?
        .to_string();
    debug!("indexing wallpaper: {}", &path_string);

    let existing = wallpapers_repository.find_by_path(&path_string)?;
    if existing.is_some() {
        return Err("wallpaper already indexed".into());
    }

    let id = Uuid::new_v7(Timestamp::now(NoContext)).to_string();
    let illustration_id = IllustrationId::from_path(path).ok();
    let tags = resolve_image_tags(path);
    let image_details = ImageDetails::read_from_path(path)?;
    let file_name = unwrap_optional_os_str(path.file_name())?;

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
    Ok(wallpaper)
}

fn resolve_image_tags(path: &Path) -> Vec<String> {
    let illustration_id = IllustrationId::from_path(path);
    if let Err(e) = illustration_id {
        debug!(
            "error resolving Pixiv illustration id for path {}: {}; ",
            path.display(),
            e
        );
        return vec![];
    }

    let tags = fetch_tags(&illustration_id.unwrap());
    if let Err(e) = tags {
        warn!(
            "failed fetching Pixiv tags for path {}: {}",
            path.display(),
            e
        );
        return vec![];
    }

    sleep(Duration::from_millis(300));
    tags.unwrap()
        .iter()
        .map(|tag| tag.translation.en.clone().trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect()
}

fn walk_directory(
    ignore_patterns: Vec<String>,
    directory: &Path,
) -> Result<impl Iterator<Item = Result<DirEntry, walkdir::Error>> + use<>, Box<dyn Error>> {
    debug!("walking directory: {}", directory.display());

    let entry_filter = move |entry: &DirEntry| {
        if entry.file_type().is_symlink() {
            debug!(
                "path excluded because is a symlink: {}",
                entry.path().display()
            );
            return false;
        }

        if !ignore_patterns.is_empty() && path_match_any_pattern(entry.path(), &ignore_patterns) {
            debug!(
                "path excluded because is ignored: {}",
                entry.path().display()
            );
            return false;
        }

        true
    };

    let walker = WalkDir::new(directory)
        .into_iter()
        .filter_entry(entry_filter);

    Ok(walker)
}
