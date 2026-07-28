use crate::wallpapers::command::WallpapersCommands;
use crate::wallpapers::command::execute_wallpaper_command;
use clap::Subcommand;
use log::warn;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    Wallpapers {
        #[command(subcommand)]
        command: WallpapersCommands,
    },
}

impl Display for Commands {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Commands::Wallpapers { command: _ } => write!(f, "wallpapers"),
        }
    }
}

pub fn execute_command(command: Commands) -> Result<(), String> {
    let result = match command.clone() {
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
