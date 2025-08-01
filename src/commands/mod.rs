pub mod wallpapers;

use clap::Subcommand;
use wallpapers::{execute_wallpapers_command, WallpapersCommand};
use crate::ApplicationContext;
use crate::config::CliConfig;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Wallpapers {
        #[command(subcommand)]
        command: WallpapersCommand,
    }
}

pub fn execute_command(command: Commands, context: &mut ApplicationContext) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Wallpapers { command } => execute_wallpapers_command(command, context),
    }
}
