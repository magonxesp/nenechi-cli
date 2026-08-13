use crate::jellyfin::config::EpisodeConfig;
use regex::Regex;

#[derive(Debug)]
pub struct EpisodePatterns {
    patterns: Vec<Regex>,
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

        Ok(Self { patterns })
    }

    /// extract the episode number from the file name and configured patterns
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
            return Ok(Some(episode));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterns(patterns: &[&str]) -> EpisodePatterns {
        EpisodePatterns::compile(&EpisodeConfig {
            patterns: patterns.iter().map(|value| value.to_string()).collect(),
        })
        .unwrap()
    }

    #[test]
    fn extracts_the_named_episode_capture() {
        let patterns = patterns(&[r"(?i)EP(?<episode>\d+)"]);
        assert_eq!(patterns.extract("Example EP12.mkv").unwrap(), Some(12));
    }
}
