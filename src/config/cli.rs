use crate::config::{DatabaseConfig, LoggingConfig};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct CliConfig {
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}

impl CliConfig {
    pub fn configure(&self) {
        self.logging.apply_configuration();
    }
}
