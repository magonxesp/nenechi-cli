use crate::jellyfin::command::{JellyfinCommands, execute_jellyfin_command};
use crate::wallpapers::command::WallpapersCommands;
use crate::wallpapers::command::execute_wallpaper_command;
use clap::Subcommand;
use log::warn;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use crate::jdownloader::{execute_jdownloader_command, JDownloaderCommands};
use crate::media::{execute_media_command, MediaCommands};

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    Jellyfin {
        #[command(subcommand)]
        command: JellyfinCommands,
    },
    Wallpapers {
        #[command(subcommand)]
        command: WallpapersCommands,
    },
    Media {
        #[command(subcommand)]
        command: MediaCommands,
    },
    #[command(name = "jdownloader")]
    JDownloader {
        #[command(subcommand)]
        command: JDownloaderCommands,
    }
}

impl Display for Commands {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Commands::Jellyfin { command: _ } => write!(f, "jellyfin"),
            Commands::Wallpapers { command: _ } => write!(f, "wallpapers"),
            Commands::Media { command: _ } => write!(f, "media"),
            Commands::JDownloader { command: _ } => write!(f, "jdownloader"),
        }
    }
}

pub fn execute_command(command: &Commands) -> Result<(), String> {
    let result = match command {
        Commands::Jellyfin {
            command: subcommand,
        } => execute_jellyfin_command(subcommand),
        Commands::Wallpapers {
            command: subcommand,
        } => execute_wallpaper_command(subcommand),
        Commands::Media {
            command: subcommand,
        } => execute_media_command(subcommand),
        Commands::JDownloader {
            command: subcommand,
        } => execute_jdownloader_command(subcommand),
    };

    if let Err(err) = result {
        return Err(format!("command {} failed: {}", command.to_string(), err));
    }

    Ok(())
}
