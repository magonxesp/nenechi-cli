use crate::config::CliConfig;
use nenechi_myanimelist::Client;
use std::sync::OnceLock;

static MY_ANIME_LIST_CLIENT: OnceLock<Client> = OnceLock::new();

pub struct MyAnimeListClient;

impl MyAnimeListClient {
    pub fn create(config: &CliConfig) -> Result<Client, String> {
        let api_key = config.myanimelist.client_id()?;
        let client = Client::new(api_key)
            .map_err(|err| format!("fallo al crear el cliente de MyAnimeList: {}", err))?;
        Ok(client)
    }

    pub fn get_instance() -> Result<Client, String> {
        if let Some(client) = MY_ANIME_LIST_CLIENT.get() {
            return Ok(client.clone());
        }

        let config = CliConfig::get_instance();
        let client = Self::create(config)?;
        match MY_ANIME_LIST_CLIENT.set(client.clone()) {
            Ok(()) => Ok(client),
            Err(client) => Ok(client),
        }
    }
}
