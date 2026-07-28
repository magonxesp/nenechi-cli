mod commands;
mod config;
mod database;
mod fs;
mod logging;
mod schema;
mod wallpapers;
mod jellyfin;

use clap::Parser;
use commands::{Commands, execute_command};
use config::read_config;
use crate::config::CliConfig;

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

    let result = execute_command(args.command);

    if let Err(err) = result {
        println!("{}", err);
        std::process::exit(1);
    }
}
