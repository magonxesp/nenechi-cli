use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
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

#[cfg(test)]
pub mod tests {
    use crate::config::DatabaseConfig;
    use std::fs;
    use std::path::Path;

    impl DatabaseConfig {
        /// Create a new database configuration for tests
        pub fn test() -> Self {
            Self {
                file: "nenechi-cli.test.db".to_string()
            }
        }

        pub fn delete_database_file(&self) {
            let path = Path::new(&self.file);

            if path.exists() {
                fs::remove_file(path).unwrap()
            }
        }
    }
}
