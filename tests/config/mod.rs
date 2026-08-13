use dotenvy::dotenv;
use nenechi_cli::config::CliConfig;
use nenechi_cli::logging;

pub fn setup() {
    dotenv().ok();

    let cli_config = cli_config();
    logging::configure(&cli_config.logging).ok();
}

pub fn cli_config() -> CliConfig {
    let mut config: CliConfig = serde_yaml::from_str(include_str!("../fixtures/config/config.yaml")).unwrap();

    // replace with environment variables secrets
    config.myanimelist.api_key = config.myanimelist.api_key.replace(
        "$MYANIMELIST_API_KEY",
        std::env::var("MYANIMELIST_API_KEY").unwrap_or("".to_string()).as_str()
    );

    config
}
