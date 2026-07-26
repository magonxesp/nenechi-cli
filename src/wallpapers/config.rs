use crate::config::resolve_configs_dir;
use log::warn;
use std::fs;
use std::path::Path;
use serde::Deserialize;

const CONFIG_FILE_NAME: &str = "wallpapers.yaml";

#[derive(Debug, Deserialize)]
struct WallpapersConfigFile {
    wallpapers: WallpapersConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WallpapersConfig {
    #[serde(default)]
    directory: String,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    tidy: Option<TidyWallpapersConfig>
}

impl Default for WallpapersConfig {
    fn default() -> Self {
        Self {
            directory: "".to_string(),
            ignore: vec![],
            tidy: None
        }
    }
}

impl WallpapersConfig {
    pub fn read() -> Result<Self, String> {
        let configs_directory = resolve_configs_dir().ok_or_else(|| {
            let message = "unable to read wallpapers configuration because the configuration directory does not exist";
            warn!("{}", message);
            message.to_string()
        })?;
        let path = configs_directory.join(CONFIG_FILE_NAME);

        if !path.is_file() {
            let message = format!(
                "unable to read wallpapers configuration because {} does not exist",
                path.display()
            );
            warn!("{}", message);
            return Err(message);
        }

        let content = fs::read_to_string(&path).map_err(|error| {
            format!(
                "failed reading wallpapers configuration {}: {}",
                path.display(),
                error
            )
        })?;
        let config: WallpapersConfigFile = serde_yaml::from_str(&content).map_err(|error| {
            format!(
                "invalid wallpapers configuration {}: {}",
                path.display(),
                error
            )
        })?;

        Ok(config.wallpapers)
    }

    pub fn directory(&self) -> Result<&Path, String> {
        if self.directory.is_empty() {
            return Err("wallpapers.directory is required".into());
        }

        let path = Path::new(&self.directory);

        if !path.is_dir() {
            return Err(format!("{} is not a directory", self.directory));
        }

        Ok(path)
    }

    pub fn tidy(&self) -> Result<TidyWallpapersConfig, String> {
        self.tidy
            .clone()
            .ok_or("wallpapers.tidy configuration is required".into())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TidyWallpapersConfig {
    #[serde(default)]
    aspect_ratio_directory: String,
    #[serde(default)]
    tags_directory: String
}

impl TidyWallpapersConfig {
    pub fn aspect_ratio_directory(&self) -> Result<&Path, String> {
        if self.aspect_ratio_directory.is_empty() {
            return Err("wallpapers.tidy.aspect_ratio_directory is required".to_string());
        }

        let path = Path::new(&self.aspect_ratio_directory);

        if path.exists() && !path.is_dir() {
            return Err(format!("{} is not a directory", self.aspect_ratio_directory));
        }

        Ok(path)
    }

    pub fn tags_directory(&self) -> Result<&Path, String> {
        if self.tags_directory.is_empty() {
            return Err("wallpapers.tidy.tags_directory is required".to_string());
        }

        let path = Path::new(&self.tags_directory);

        if path.exists() && !path.is_dir() {
            return Err(format!("{} is not a directory", self.tags_directory));
        }

        Ok(path)
    }
}
