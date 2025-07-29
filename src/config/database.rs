use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "DatabaseConfig::default_file")]
    file: String,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            file: DatabaseConfig::default_file(),
        }
    }
}

impl DatabaseConfig {
    fn default_file() -> String {
        String::from("nenechi-cli.db")
    }

    pub fn sqlite_uri(&self) -> String {
        format!("sqlite://{}", self.file)
    }
}
