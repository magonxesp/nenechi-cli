use clap::Parser;
use nenechi_cli::config::{CliConfig};
use nenechi_cli::commands::{Commands, execute_command};
use nenechi_cli::logging;

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
