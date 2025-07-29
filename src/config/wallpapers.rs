use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
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

    pub fn tidy(&self) -> Result<&TidyWallpapersConfig, String> {
        self.tidy
            .as_ref()
            .ok_or("wallpapers.tidy configuration is required".into())
    }
}

#[derive(Debug, Deserialize)]
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

        if !path.is_dir() {
            return Err(format!("{} is not a directory", self.aspect_ratio_directory));
        }

        Ok(path)
    }

    pub fn tags_directory(&self) -> Result<&Path, String> {
        if self.tags_directory.is_empty() {
            return Err("wallpapers.tidy.tags_directory is required".to_string());
        }

        let path = Path::new(&self.tags_directory);

        if !path.is_dir() {
            return Err(format!("{} is not a directory", self.tags_directory));
        }

        Ok(path)
    }
}
