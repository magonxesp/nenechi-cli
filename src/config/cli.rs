use std::sync::OnceLock;
use crate::config::{read_config, DatabaseConfig, LoggingConfig};
use serde::Deserialize;

static CLI_CONFIG_INSTANCE: OnceLock<CliConfig> = OnceLock::new();

#[derive(Debug, Deserialize, Clone)]
pub struct CliConfig {
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
}

impl CliConfig {
    pub fn get_instance<'a>() -> &'a Self {
        CLI_CONFIG_INSTANCE.get_or_init(|| read_config())
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            database: DatabaseConfig::default(),
        }
    }
}
