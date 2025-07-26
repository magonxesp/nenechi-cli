mod commands;

use clap::Parser;
use commands::Commands;
use commands::execute_command;

#[derive(Debug, Parser)]
#[command(name = "nennechi-cli")]
#[command(about = "Utils para el servidor nenechi", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let args = Cli::parse();

    execute_command(args.command).unwrap();
}
