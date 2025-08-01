use std::fs;

use serde::Deserialize;
use crate::config::{DatabaseConfig, LoggingConfig, WallpapersConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct CliConfig {
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub wallpapers: WallpapersConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            wallpapers: WallpapersConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}

impl CliConfig {
    pub fn read(path: &str) -> Self {
        let exists = fs::exists(path).unwrap();

        if !exists {
            dbg!("Using default configuration because the configuration file {} does not exist", path);
            return CliConfig::default();
        }

        let content = fs::read_to_string(&path).unwrap();
        serde_yaml::from_str(&content).unwrap()
    }

    pub fn configure(&self) {
        self.logging.apply_configuration();
    }
}
