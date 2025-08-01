mod config;
mod commands;
mod models;
mod schema;
mod fs;
mod database;

use crate::database::create_db_connection;
use clap::Parser;
use commands::execute_command;
use commands::Commands;
use config::CliConfig;
use diesel::SqliteConnection;

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

struct ApplicationContext<'a> {
    config: CliConfig,
    db_connection: &'a mut SqliteConnection,
}

fn main() {
    let args = Cli::parse();
    let config = CliConfig::read(args.config_file.as_str());
    config.configure();

    let mut db_connection = create_db_connection(&config.database);
    let mut context = ApplicationContext {
        config,
        db_connection: &mut db_connection,
    };

    execute_command(args.command, &mut context).unwrap();
}
