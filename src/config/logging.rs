use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub enum LoggingOutput {
    #[serde(rename = "console")]
    Console,
    #[serde(rename = "file")]
    File,
}

impl Default for LoggingOutput {
    fn default() -> Self {
        LoggingOutput::Console
    }
}

#[derive(Debug, Deserialize)]
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
    pub fn as_log4rs_enum(&self) -> LevelFilter {
        match self {
            LoggingLevel::Debug => LevelFilter::Debug,
            LoggingLevel::Info => LevelFilter::Info,
            LoggingLevel::Warn => LevelFilter::Warn,
            LoggingLevel::Error => LevelFilter::Error,
            LoggingLevel::Trace => LevelFilter::Trace
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    #[serde(default)]
    pub level: LoggingLevel,
    #[serde(default)]
    pub output: LoggingOutput,
    #[serde(default = "LoggingConfig::default_file")]
    pub file: String
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: LoggingLevel::default(),
            output: LoggingOutput::default(),
            file: LoggingConfig::default_file()
        }
    }
}

impl LoggingConfig {
    pub fn apply_configuration(&self) {
        let config = match self.output {
            LoggingOutput::Console => self.configure_console_logger(),
            LoggingOutput::File => self.configure_file_logger(),
        };

        log4rs::init_config(config).unwrap();
    }

    fn configure_console_logger(&self) -> log4rs::Config {
        let stdout = ConsoleAppender::builder().build();

        let appender = Appender::builder()
            .build("stdout", Box::new(stdout));

        let root = Root::builder()
            .appender("stdout")
            .build(self.level.as_log4rs_enum());

        Config::builder()
            .appender(appender)
            .build(root)
            .unwrap()
    }

    fn configure_file_logger(&self) -> log4rs::Config {
        let file = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
            .build(&self.file)
            .unwrap();

        let appender = Appender::builder()
            .build("file", Box::new(file));

        let root = Root::builder()
            .appender("file")
            .build(self.level.as_log4rs_enum());

        Config::builder()
            .appender(appender)
            .build(root)
            .unwrap()
    }

    fn default_file() -> String {
        String::from("nenechi-cli.log")
    }
}
