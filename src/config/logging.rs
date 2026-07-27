use log::LevelFilter;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum LoggingLevel {
    #[serde(rename = "trace")]
    Trace,
    #[serde(rename = "debug")]
    Debug,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "error")]
    Error,
}

impl Default for LoggingLevel {
    fn default() -> Self {
        LoggingLevel::Info
    }
}

impl LoggingLevel {
    pub fn as_level_filter(&self) -> LevelFilter {
        match self {
            LoggingLevel::Debug => LevelFilter::Debug,
            LoggingLevel::Info => LevelFilter::Info,
            LoggingLevel::Warn => LevelFilter::Warn,
            LoggingLevel::Error => LevelFilter::Error,
            LoggingLevel::Trace => LevelFilter::Trace,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    #[serde(default)]
    pub level: LoggingLevel,
    #[serde(default = "LoggingConfig::default_file")]
    pub file: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: LoggingLevel::default(),
            file: LoggingConfig::default_file(),
        }
    }
}

impl LoggingConfig {
    fn default_file() -> String {
        String::from("nenechi-cli.log")
    }
}
