use crate::jellyfin::config::{EpisodeConfig, EpisodeFallback};
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumberedEpisode {
    pub path: PathBuf,
    pub number: u32,
}

#[derive(Debug)]
pub struct EpisodePatterns {
    patterns: Vec<Regex>,
    fallback: EpisodeFallback,
    start_at: u32,
}

impl EpisodePatterns {
    pub fn compile(config: &EpisodeConfig) -> Result<Self, String> {
        let mut patterns = Vec::with_capacity(config.patterns.len());

        for value in &config.patterns {
            let pattern = Regex::new(value)
                .map_err(|error| format!("invalid episode pattern {value:?}: {error}"))?;
            let has_episode_capture = pattern
                .capture_names()
                .flatten()
                .any(|name| name == "episode");

            if !has_episode_capture {
                return Err(format!(
                    "episode pattern {value:?} must contain a named capture group called \"episode\""
                ));
            }
            patterns.push(pattern);
        }

        Ok(Self {
            patterns,
            fallback: config.fallback.clone(),
            start_at: config.start_at,
        })
    }

    pub fn extract(&self, file_name: &str) -> Result<Option<u32>, String> {
        for pattern in &self.patterns {
            let Some(captures) = pattern.captures(file_name) else {
                continue;
            };
            let value = captures
                .name("episode")
                .ok_or_else(|| {
                    format!("episode capture did not participate when matching file {file_name:?}")
                })?
                .as_str();
            let episode = value.parse::<u32>().map_err(|error| {
                format!("invalid episode number {value:?} in file {file_name:?}: {error}")
            })?;
            if episode == 0 {
                return Err(format!(
                    "episode number in file {file_name:?} cannot be zero"
                ));
            }
            return Ok(Some(episode));
        }

        Ok(None)
    }

    pub fn number_files(&self, files: &[PathBuf]) -> Result<Vec<NumberedEpisode>, String> {
        let mut extracted = Vec::with_capacity(files.len());
        let mut all_matched = true;

        for path in files {
            let file_name = path
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| format!("file name {} is not valid UTF-8", path.display()))?;
            let number = self.extract(file_name)?;
            all_matched &= number.is_some();
            extracted.push((path.clone(), number));
        }

        if all_matched {
            let numbered = extracted
                .into_iter()
                .map(|(path, number)| NumberedEpisode {
                    path,
                    number: number.expect("all episode numbers were checked above"),
                })
                .collect::<Vec<_>>();
            validate_unique_episodes(&numbered)?;
            return Ok(numbered);
        }

        match self.fallback {
            EpisodeFallback::FilesystemOrder => files
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, path)| {
                    let offset = u32::try_from(index)
                        .map_err(|_| "too many episode files to number".to_string())?;
                    let number = self.start_at.checked_add(offset).ok_or_else(|| {
                        "episode number overflow while applying filesystem-order fallback"
                            .to_string()
                    })?;
                    Ok(NumberedEpisode { path, number })
                })
                .collect(),
        }
    }
}

fn validate_unique_episodes(episodes: &[NumberedEpisode]) -> Result<(), String> {
    let mut seen = HashSet::new();
    for episode in episodes {
        if !seen.insert(episode.number) {
            return Err(format!(
                "episode {} was extracted from more than one file",
                episode.number
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterns(patterns: &[&str]) -> EpisodePatterns {
        EpisodePatterns::compile(&EpisodeConfig {
            patterns: patterns.iter().map(|value| value.to_string()).collect(),
            fallback: EpisodeFallback::FilesystemOrder,
            start_at: 1,
        })
        .unwrap()
    }

    #[test]
    fn extracts_the_named_episode_capture() {
        let patterns = patterns(&[r"(?i)EP(?<episode>\d+)"]);
        assert_eq!(patterns.extract("Example EP12.mkv").unwrap(), Some(12));
    }

    #[test]
    fn fallback_preserves_the_received_filesystem_order() {
        let patterns = patterns(&[r"EP(?<episode>\d+)"]);
        let files = vec![
            PathBuf::from("show 10.mkv"),
            PathBuf::from("show 2.mkv"),
            PathBuf::from("show 1.mkv"),
        ];

        let numbered = patterns.number_files(&files).unwrap();
        let names = numbered
            .iter()
            .map(|episode| {
                episode
                    .path
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect::<Vec<_>>();

        assert_eq!(names, ["show 10.mkv", "show 2.mkv", "show 1.mkv"]);
        assert_eq!(
            numbered
                .iter()
                .map(|episode| episode.number)
                .collect::<Vec<_>>(),
            [1, 2, 3]
        );
    }

    #[test]
    fn rejects_duplicate_extracted_episode_numbers() {
        let patterns = patterns(&[r"EP(?<episode>\d+)"]);
        let files = vec![PathBuf::from("a EP1.mkv"), PathBuf::from("b EP1.mkv")];
        assert!(patterns.number_files(&files).is_err());
    }
}
