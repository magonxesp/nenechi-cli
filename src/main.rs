mod config;
mod commands;
mod models;
mod schema;

use clap::Parser;
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use commands::Commands;
use commands::execute_command;

use crate::config::CliConfig;

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

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

fn main() {
    let args = Cli::parse();
    let config = CliConfig::read(args.config_file.as_str());
    config.configure();

    let mut db_connection = SqliteConnection::establish(&config.database.sqlite_uri())
        .unwrap_or_else(|_| panic!("Error connecting to {}", config.database.sqlite_uri()));

    db_connection.run_pending_migrations(MIGRATIONS)
        .expect("Error running migrations");

    execute_command(args.command, &config).unwrap();
}
