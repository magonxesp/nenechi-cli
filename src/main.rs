mod config;
mod commands;

use clap::Parser;
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

fn main() {
    let args = Cli::parse();
    let config = CliConfig::read(args.config_file.as_str());

    config.configure();

    execute_command(args.command).unwrap();
}
