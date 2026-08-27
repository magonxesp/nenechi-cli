use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct MediaAnimeConfig {
    #[serde(default = "MediaAnimeConfig::default_directory")]
    pub directory: String,
    #[serde(default = "MediaAnimeConfig::default_episode_whitelist")]
    pub episodes_whitelist: Vec<String>,
}

impl Default for MediaAnimeConfig {
    fn default() -> Self {
        Self {
            directory: Self::default_directory(),
            episodes_whitelist: Self::default_episode_whitelist(),
        }
    }
}

impl MediaAnimeConfig {
    fn default_directory() -> String {
        String::from("")
    }

    fn default_episode_whitelist() -> Vec<String> {
        Vec::new()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct MediaConfig {
    #[serde(default)]
    pub anime: MediaAnimeConfig,
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            anime: MediaAnimeConfig::default(),
        }
    }
}
