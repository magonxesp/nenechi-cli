pub mod tidy_wallpapers;

use clap::Subcommand;
use tidy_wallpapers::tidy_wallpapers;

use crate::config::CliConfig;

#[derive(Debug, Subcommand)]
pub enum Commands {
    TidyWallpapers
}

pub fn execute_command(command: Commands, config: &CliConfig) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::TidyWallpapers => tidy_wallpapers(
            config.tidy_wallpapers
                .as_ref()
                .ok_or("tidy_wallpapers is not configured")?
        ),
    }
}
