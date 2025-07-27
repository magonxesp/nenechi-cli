use std::path::Path;

use clap::Subcommand;
use log::{debug, info};
use serde::Deserialize;
use std::fs;
use walkdir::WalkDir;

#[derive(Debug, Subcommand)]
pub enum WallpapersCommand {
    Tidy,
    Index,
    CleanIndex
}

#[derive(Debug, Deserialize)]
pub struct WallpapersConfig {
    #[serde(default)]
    tidy: Option<TidyWallpapersConfig>
}

impl Default for WallpapersConfig {
    fn default() -> Self {
        Self {
            tidy: None
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TidyWallpapersConfig {
    #[serde(default)]
    all_directory: String,
    #[serde(default)]
    aspect_ratio_directory: String,
    #[serde(default)]
    tags_directory: String
}

impl TidyWallpapersConfig {
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.all_directory.is_empty() {
            return Err("all_directory is required".into());
        }

        if self.all_directory.is_empty() {
            return Err("all_directory is required".into());
        }

        if self.aspect_ratio_directory.is_empty() {
            return Err("aspect_ratio_directory is required".into());
        }

        if self.tags_directory.is_empty() {
            return Err("tags_directory is required".into());
        }

        Ok(())
    }

    pub fn all_directory(&self) -> Result<&Path, String> {
        let path = Path::new(&self.all_directory);

        if !path.is_dir() {
            return Err(format!("{} is not a directory", self.all_directory));
        }

        Ok(path)
    }

    pub fn aspect_ratio_directory(&self) -> Result<&Path, String> {
        let path = Path::new(&self.aspect_ratio_directory);

        if !path.is_dir() {
            return Err(format!("{} is not a directory", self.aspect_ratio_directory));
        }

        Ok(path)
    }

    pub fn tags_directory(&self) -> Result<&Path, String> {
        let path = Path::new(&self.tags_directory);

        if !path.is_dir() {
            return Err(format!("{} is not a directory", self.tags_directory));
        }

        Ok(path)
    }
}

pub fn execute_wallpapers_command(
    command: WallpapersCommand,
    config: &WallpapersConfig
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        WallpapersCommand::Tidy => tidy_wallpapers(
            config.tidy
                .as_ref()
                .ok_or("wallpapers.tidy configuration is required")?
        ),
        WallpapersCommand::Index => Err("Not implemented".into()),
        WallpapersCommand::CleanIndex => Err("Not implemented".into())
    }
}

pub fn tidy_wallpapers(config: &TidyWallpapersConfig) -> Result<(), Box<dyn std::error::Error>> {
    debug!("using config: {:?}", config);

    config.validate()?;

    info!("tidying wallpapers: {}", config.all_directory);

    for entry in WalkDir::new(config.all_directory()?) {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_file() && !metadata.is_symlink() {
            info!("Archivo: {}", path.display());
        }
    }

    Ok(())
}
