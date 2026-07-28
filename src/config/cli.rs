use crate::config::{DatabaseConfig, LoggingConfig, MyAnimeListConfig, read_config};
use serde::Deserialize;
use std::sync::OnceLock;

static CLI_CONFIG_INSTANCE: OnceLock<CliConfig> = OnceLock::new();

#[derive(Debug, Deserialize, Clone)]
pub struct CliConfig {
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub myanimelist: MyAnimeListConfig,
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
            myanimelist: MyAnimeListConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_myanimelist_client_id_from_example_config() {
        let config: CliConfig =
            serde_yaml::from_str(include_str!("../../examples/config.yaml")).unwrap();

        assert_eq!(config.myanimelist.api_key, "your-client-id");
    }
}
