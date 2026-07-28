use crate::fs::expand_user_dir;
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    #[serde(default = "DatabaseConfig::default_file")]
    pub file: String,
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
        format!("sqlite://{}", expand_user_dir(&self.file).to_string_lossy())
    }

    pub fn directory(&self) -> Option<PathBuf> {
        let path = Path::new(self.file.as_str());

        if let Some(directory) = path.parent() {
            let directory = directory.to_path_buf();
            Some(expand_user_dir(directory))
        } else {
            None
        }
    }

    pub fn filename(&self) -> Option<PathBuf> {
        let path = Path::new(self.file.as_str());
        path.file_name().map(|s| s.into())
    }

    pub fn path(&self) -> Option<PathBuf> {
        let filename = self.filename()?;

        if let Some(directory) = self.directory() {
            Some(directory.join(filename))
        } else {
            None
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::config::DatabaseConfig;

    impl DatabaseConfig {
        /// Create a new database configuration for tests
        pub fn test() -> Self {
            Self {
                file: "nenechi-cli.test.db".to_string(),
            }
        }
    }
}
