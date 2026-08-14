use crate::jellyfin::config::{JellyfinConfig, TargetType};
use crate::jellyfin::{movies, series};
use clap::Subcommand;
use log::{info, warn};
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Subcommand)]
pub enum JellyfinCommands {
    Mount,
}

impl Display for JellyfinCommands {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mount => formatter.write_str("mount"),
        }
    }
}

pub fn execute_jellyfin_command(command: &JellyfinCommands) -> Result<(), String> {
    let config = JellyfinConfig::read()?;
    let result = match command {
        JellyfinCommands::Mount => mount(&config),
    };

    result
        .map(|_| ())
        .map_err(|error| format!("subcommand {} failed: {error}", command))
}

fn mount(config: &JellyfinConfig) -> Result<usize, String> {
    let mut links = 0;

    for target in &config.targets {
        let result = match &target.target_type {
            TargetType::Series => series::organize(target),
            TargetType::Movies => movies::organize(target),
        };

        match result {
            Ok(target_links) => links += target_links,
            Err(error) => warn!(
                "failed mounting Jellyfin target {:?} at {}: {error}",
                target.name,
                target.destination.display()
            ),
        }
    }

    info!("finished mounting Jellyfin structure: {links} symbolic links");
    Ok(links)
}
