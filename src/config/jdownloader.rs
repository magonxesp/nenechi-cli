use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct JDownloaderConfig {
    #[serde(default = "JDownloaderConfig::default_api_base_url")]
    pub api_base_url: String,
}

impl Default for JDownloaderConfig {
    fn default() -> Self {
        Self {
            api_base_url: Self::default_api_base_url(),
        }
    }
}

impl JDownloaderConfig {
    fn default_api_base_url() -> String {
        String::from("http://localhost:3128")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_example_api_base_url_by_default() {
        assert_eq!(
            JDownloaderConfig::default().api_base_url,
            "http://localhost:3128"
        );
    }
}
