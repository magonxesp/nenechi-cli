mod tidy_wallpapers;

use clap::Subcommand;
use tidy_wallpapers::tidy_wallpapers;

#[derive(Debug, Subcommand)]
pub enum Commands {
    TidyWallpapers {
        #[arg(required = true)]
        path: String
    }
}

pub fn execute_command(command: Commands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::TidyWallpapers { path } => tidy_wallpapers(path.as_str()),
    }
}
