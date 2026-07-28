use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
pub struct MyAnimeListConfig {
    /// MyAnimeList API v2 Client ID.
    #[serde(default)]
    pub api_key: String,
}

impl MyAnimeListConfig {
    pub fn client_id(&self) -> Result<&str, String> {
        let client_id = self.api_key.trim();
        if client_id.is_empty() {
            return Err("myanimelist.api_key must contain the MyAnimeList API v2 Client ID".into());
        }

        Ok(client_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_empty_client_id() {
        assert!(MyAnimeListConfig::default().client_id().is_err());
    }
}
