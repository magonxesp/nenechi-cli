mod commands;
mod config;
mod database;
mod fs;
mod schema;
mod wallpapers;

use crate::database::create_db_connection;
use clap::Parser;
use commands::{Commands, execute_command};
use config::read_config;

#[derive(Debug, Parser)]
#[command(name = "nennechi-cli")]
#[command(about = "Utils para el servidor nenechi", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Cli::parse();
    let config = read_config();
    config.configure();

    let connection_pool = create_db_connection(&config.database);

    let result = execute_command(args.command, connection_pool);

    if let Err(err) = result {
        println!("{}", err);
        std::process::exit(1);
    }
}
