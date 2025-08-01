pub mod wallpapers;

use crate::commands::wallpapers::execute_wallpaper_command;
use crate::config::CliConfig;
use clap::Subcommand;
use diesel::SqliteConnection;
use wallpapers::WallpapersCommands;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Wallpapers {
        #[command(subcommand)]
        command: WallpapersCommands,
    }
}

pub fn execute_command(
    command: Commands,
    config: &CliConfig,
    db_connection: &mut SqliteConnection
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Wallpapers { command } => execute_wallpaper_command(
            command,
            config.wallpapers.clone(),
            db_connection
        ),
    }
}
