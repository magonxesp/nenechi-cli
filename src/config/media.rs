use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct MediaConfig {
    #[serde(default = "MediaConfig::default_anime_directory")]
    pub anime_directory: String,
}

impl Default for MediaConfig {
    fn default() -> Self {
        Self {
            anime_directory: Self::default_anime_directory(),
        }
    }
}

impl MediaConfig {
    fn default_anime_directory() -> String {
        String::from("")
    }
}
