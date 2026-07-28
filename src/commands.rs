use crate::jellyfin::command::{JellyfinCommands, execute_jellyfin_command};
use crate::wallpapers::command::WallpapersCommands;
use crate::wallpapers::command::execute_wallpaper_command;
use clap::Subcommand;
use log::warn;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

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
}

impl Display for Commands {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Commands::Jellyfin { command: _ } => write!(f, "jellyfin"),
            Commands::Wallpapers { command: _ } => write!(f, "wallpapers"),
        }
    }
}

pub fn execute_command(command: Commands) -> Result<(), String> {
    let result = match command.clone() {
        Commands::Jellyfin {
            command: subcommand,
        } => execute_jellyfin_command(subcommand),
        Commands::Wallpapers {
            command: subcommand,
        } => execute_wallpaper_command(subcommand),
    };

    if let Err(err) = result {
        warn!("command {} failed: {}", command.to_string(), err);
        return Err(err.to_string());
    }

    Ok(())
}
