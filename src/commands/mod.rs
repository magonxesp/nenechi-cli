pub mod wallpapers;

use crate::commands::wallpapers::execute_wallpaper_command;
use crate::config::CliConfig;
use clap::Subcommand;
use diesel::r2d2::{ConnectionManager, Pool};
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
    config: CliConfig,
    connection_pool: Pool<ConnectionManager<SqliteConnection>>
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Wallpapers { command } => execute_wallpaper_command(
            command,
            config.wallpapers.clone(),
            connection_pool
        ),
    }
}
