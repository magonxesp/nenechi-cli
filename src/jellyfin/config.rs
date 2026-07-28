use crate::config::resolve_configs_dir;
use glob::Pattern as GlobPattern;
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "jellyfin.yaml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JellyfinConfigRoot {
    jellyfin: JellyfinConfig,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JellyfinConfig {
    #[serde(default)]
    pub targets: Vec<TargetConfig>,
}

impl JellyfinConfig {
    pub fn read() -> Result<Self, String> {
        let configs_directory = resolve_configs_dir().ok_or_else(|| {
            "unable to read Jellyfin configuration because the configuration directory does not exist"
                .to_string()
        })?;
        let path = configs_directory.join(CONFIG_FILE_NAME);

        if !path.is_file() {
            return Err(format!(
                "unable to read Jellyfin configuration because {} does not exist",
                path.display()
            ));
        }

        let content = fs::read_to_string(&path).map_err(|error| {
            format!(
                "failed reading Jellyfin configuration {}: {error}",
                path.display()
            )
        })?;
        let root: JellyfinConfigRoot = serde_yaml::from_str(&content).map_err(|error| {
            format!("invalid Jellyfin configuration {}: {error}", path.display())
        })?;

        root.jellyfin.validate()?;
        Ok(root.jellyfin)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.targets.is_empty() {
            return Err("jellyfin.targets must contain at least one target".into());
        }

        let mut names = HashSet::new();
        for target in &self.targets {
            target.validate()?;
            if !names.insert(target.name.as_str()) {
                return Err(format!(
                    "jellyfin target name {:?} is duplicated",
                    target.name
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TargetType {
    Series,
    Movies,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub target_type: TargetType,
    pub source: PathBuf,
    pub destination: PathBuf,
    #[serde(default)]
    pub series: Option<SeriesConfig>,
    #[serde(default = "default_include_patterns")]
    pub include: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
}

impl TargetConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("jellyfin target name cannot be empty".into());
        }
        if self.source.as_os_str().is_empty() {
            return Err(format!(
                "jellyfin target {:?} source is required",
                self.name
            ));
        }
        if self.destination.as_os_str().is_empty() {
            return Err(format!(
                "jellyfin target {:?} destination is required",
                self.name
            ));
        }
        if self.include.is_empty() {
            return Err(format!(
                "jellyfin target {:?} include must contain at least one pattern",
                self.name
            ));
        }

        for pattern in &self.include {
            GlobPattern::new(pattern).map_err(|error| {
                format!(
                    "invalid include pattern {:?} in Jellyfin target {:?}: {error}",
                    pattern, self.name
                )
            })?;
        }
        for pattern in &self.ignore {
            GlobPattern::new(pattern).map_err(|error| {
                format!(
                    "invalid ignore pattern {:?} in Jellyfin target {:?}: {error}",
                    pattern, self.name
                )
            })?;
        }

        match (&self.target_type, &self.series) {
            (TargetType::Series, Some(series)) => series.validate(&self.name),
            (TargetType::Series, None) => Err(format!(
                "jellyfin target {:?} requires series configuration",
                self.name
            )),
            (TargetType::Movies, Some(_)) => Err(format!(
                "jellyfin movies target {:?} cannot contain series configuration",
                self.name
            )),
            (TargetType::Movies, None) => Ok(()),
        }
    }

    pub fn validate_source(&self) -> Result<(), String> {
        if !self.source.is_dir() {
            return Err(format!(
                "Jellyfin target {:?} source {} is not a directory",
                self.name,
                self.source.display()
            ));
        }
        if self.destination.exists() && !self.destination.is_dir() {
            return Err(format!(
                "Jellyfin target {:?} destination {} is not a directory",
                self.name,
                self.destination.display()
            ));
        }
        Ok(())
    }

    pub(crate) fn includes(&self, path: &Path) -> bool {
        let relative = path.strip_prefix(&self.source).unwrap_or(path);
        self.include.iter().any(|value| {
            let Ok(pattern) = GlobPattern::new(value) else {
                return false;
            };
            pattern.matches_path(relative)
                || path
                    .file_name()
                    .is_some_and(|file_name| pattern.matches_path(Path::new(file_name)))
        })
    }

    pub(crate) fn ignores(&self, path: &Path) -> bool {
        let relative = path.strip_prefix(&self.source).unwrap_or(path);
        self.ignore.iter().any(|value| {
            GlobPattern::new(value).is_ok_and(|pattern| pattern.matches_path(relative))
        })
    }
}

fn default_include_patterns() -> Vec<String> {
    vec!["*.mkv".into(), "*.mp4".into(), "*.avi".into()]
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeriesConfig {
    pub category: SeriesCategory,
    pub episode: EpisodeConfig,
}

impl SeriesConfig {
    fn validate(&self, target_name: &str) -> Result<(), String> {
        self.episode.validate(target_name)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SeriesCategory {
    Anime,
    LiveAction,
    Animation,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeConfig {
    #[serde(default)]
    pub patterns: Vec<String>,
    pub fallback: EpisodeFallback,
    #[serde(default = "default_episode_start")]
    pub start_at: u32,
}

impl EpisodeConfig {
    fn validate(&self, target_name: &str) -> Result<(), String> {
        if self.start_at == 0 {
            return Err(format!(
                "jellyfin target {:?} series.episode.start_at must be greater than zero",
                target_name
            ));
        }
        crate::jellyfin::pattern::EpisodePatterns::compile(self)
            .map(|_| ())
            .map_err(|error| format!("jellyfin target {:?}: {error}", target_name))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeFallback {
    FilesystemOrder,
}

fn default_episode_start() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_example_shape() {
        let root: JellyfinConfigRoot =
            serde_yaml::from_str(include_str!("../../examples/conf.d/jellyfin.yaml")).unwrap();
        root.jellyfin.validate().unwrap();
        assert_eq!(root.jellyfin.targets[0].target_type, TargetType::Series);
        assert!(!root.jellyfin.targets[0].ignore.is_empty());
        assert!(root.jellyfin.targets[1].ignore.is_empty());
        assert!(
            root.jellyfin.targets[0].ignores(Path::new("/mnt/downloads/anime/Ignored Example"))
        );
    }

    #[test]
    fn rejects_an_episode_pattern_without_the_named_capture() {
        let episode = EpisodeConfig {
            patterns: vec![r"EP(\d+)".into()],
            fallback: EpisodeFallback::FilesystemOrder,
            start_at: 1,
        };

        assert!(episode.validate("anime").is_err());
    }

    #[test]
    fn rejects_an_invalid_ignore_glob() {
        let mut root: JellyfinConfigRoot =
            serde_yaml::from_str(include_str!("../../examples/conf.d/jellyfin.yaml")).unwrap();
        root.jellyfin.targets[0].ignore = vec!["[".into()];

        assert!(root.jellyfin.validate().is_err());
    }
}
