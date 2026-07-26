use crate::config::CliConfig;
use std::env;
use std::fs;
use std::path::PathBuf;

const CONFIG_FILE_NAME: &str = "config.yaml";

/// Resolves the first standard configuration directory containing `config.yaml`.
///
/// The directories are checked in this order:
/// `~/.nenechi`, `~/.config/nenechi`, and `/etc/nenechi` on Linux.
pub fn resolve_config_dir() -> Option<PathBuf> {
    let mut directories = Vec::new();

    if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        directories.push(home.join(".nenechi"));
        directories.push(home.join(".config").join("nenechi"));
    }

    directories.push(PathBuf::from("/etc/nenechi"));
    find_config_dir(&directories)
}

/// Resolves the directory containing isolated command configurations.
pub fn resolve_configs_dir() -> Option<PathBuf> {
    resolve_config_dir().map(|config_directory| config_directory.join("conf.d"))
}

/// Discovers and reads `config.yaml` from a standard configuration directory.
pub fn read_config() -> CliConfig {
    let path = resolve_config_dir().map(|directory| directory.join(CONFIG_FILE_NAME));

    let Some(path) = path else {
        eprintln!("Configuration file not found; using default configuration");
        return CliConfig::default();
    };

    if !path.is_file() {
        eprintln!(
            "Configuration file {} does not exist; using default configuration",
            path.display()
        );
        return CliConfig::default();
    }

    let content = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed reading configuration file {}: {error}",
            path.display()
        )
    });

    serde_yaml::from_str(&content)
        .unwrap_or_else(|error| panic!("invalid configuration file {}: {error}", path.display()))
}

fn find_config_dir(directories: &[PathBuf]) -> Option<PathBuf> {
    for directory in directories {
        let config_file = directory.join(CONFIG_FILE_NAME);

        if config_file.is_file() {
            return Some(directory.to_path_buf());
        }
    }

    None
}
