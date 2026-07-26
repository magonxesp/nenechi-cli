mod config;
mod commands;
mod schema;
mod fs;
mod database;
mod wallpapers;

use crate::database::create_db_connection;
use clap::Parser;
use commands::{Commands, execute_command};
use config::CliConfig;

#[derive(Debug, Parser)]
#[command(name = "nennechi-cli")]
#[command(about = "Utils para el servidor nenechi", long_about = None)]
struct Cli {
    #[arg(
        short, 
        long, 
        help = "Configuration file path", 
        default_value = "config.yml"
    )]
    config_file: String,

    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Cli::parse();
    let config = CliConfig::read(args.config_file.as_str());
    config.configure();

    let connection_pool = create_db_connection(&config.database);

    let result = execute_command(
        args.command,
        config,
        connection_pool
    );

    if let Err(err) = result {
        println!("{}", err);
        std::process::exit(1);
    }
}
