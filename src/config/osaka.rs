use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct OsakaConfig {
    #[serde(default = "OsakaConfig::default_base_url")]
    pub base_url: String,
}

impl Default for OsakaConfig {
    fn default() -> Self {
        Self {
            base_url: Self::default_base_url(),
        }
    }
}

impl OsakaConfig {
    fn default_base_url() -> String {
        String::from("http://localhost:8080")
    }
}
