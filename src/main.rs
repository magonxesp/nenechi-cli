mod anime;
mod client;
mod commands;
mod config;
mod database;
mod fs;
mod jellyfin;
mod logging;
mod schema;
mod wallpapers;
mod media;
mod jdownloader;
mod osaka;

use clap::Parser;
use log::warn;
use config::{CliConfig};
use commands::{Commands, execute_command};

#[derive(Debug, Parser)]
#[command(name = "nennechi-cli")]
#[command(about = "Utils para el servidor nenechi", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Cli::parse();
    let config = CliConfig::get_instance();
    logging::configure(&config.logging).expect("failed to configure logging");

    let result = execute_command(&args.command);

    if let Err(err) = result {
        warn!("{}", err);
        std::process::exit(1);
    }
}
