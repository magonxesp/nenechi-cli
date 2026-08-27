use crate::config::CliConfig;
use clap::Subcommand;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use crate::anime::download::AnimeDownloader;
use crate::jdownloader::{JDownloader, JobId};

#[derive(Clone, Debug, Subcommand)]
pub enum AnimeCommands {
    Download {
        url: String,

        #[arg(short, long)]
        incremental: bool,
    }
}

impl Display for AnimeCommands {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Download { .. } => formatter.write_str("download"),
        }
    }
}

pub fn execute_anime_command(command: &AnimeCommands) -> Result<(), String> {
    let result = match command {
        AnimeCommands::Download { url, incremental } => download(url, incremental),
    };

    result
        .map(|_| ())
        .map_err(|error| format!("subcommand {} failed: {error}", command))
}

fn download(url: &String, incremental: &bool) -> Result<(), String> {
    let config = CliConfig::get_instance();
    let downloader = AnimeDownloader::from_config(config)
        .map_err(|e| format!("failed to create anime downloader: {}", e))?;

    downloader.download(url, *incremental).map_err(|error| format!("downloader failed: {error}"))
}
