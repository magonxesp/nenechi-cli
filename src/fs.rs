use glob::Pattern;
use log::{info, warn};
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn expand_user_dir(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();

    let Some(relative_path) = path.strip_prefix("~").ok() else {
        return path.to_path_buf();
    };

    let Some(home_dir) = env::var_os("HOME") else {
        return path.to_path_buf();
    };

    PathBuf::from(home_dir).join(relative_path)
}

pub fn path_match_any_pattern(path: &Path, patterns: &[String]) -> bool {
    let mut valid_patterns = vec![];

    for pattern in patterns {
        match Pattern::new(pattern.as_str()) {
            Ok(pattern) => valid_patterns.push(pattern),
            Err(_) => warn!(
                "ignore pattern {} is not valid and will not be applied",
                pattern
            ),
        }
    }

    for pattern in valid_patterns {
        if pattern.matches_path(path) {
            return true;
        }
    }

    false
}

pub fn unwrap_optional_os_str(os_str: Option<&OsStr>) -> Result<String, Box<dyn Error>> {
    let path_str = os_str.ok_or("os string is none")?.to_str();
    let path_string = path_str.ok_or("str is none")?.to_string();

    Ok(path_string)
}

pub fn sanitize_filename_component(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());

    for character in value.chars() {
        let invalid = character.is_control()
            || matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            );
        if invalid {
            if !sanitized.ends_with('_') {
                sanitized.push('_');
            }
        } else {
            sanitized.push(character);
        }
    }

    let sanitized = sanitized
        .trim()
        .trim_end_matches(&['.', ' '][..])
        .to_string();
    let mut sanitized = if sanitized.is_empty() {
        "_".to_string()
    } else {
        sanitized
    };

    if is_windows_reserved_name(&sanitized) {
        sanitized.insert(0, '_');
    }

    sanitized
}

pub fn sanitize_relative_path(path: &Path) -> PathBuf {
    path.components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => {
                Some(sanitize_filename_component(&value.to_string_lossy()))
            }
            _ => None,
        })
        .collect()
}

fn is_windows_reserved_name(value: &str) -> bool {
    let stem = value
        .split('.')
        .next()
        .unwrap_or(value)
        .trim_end_matches(&['.', ' '][..])
        .to_ascii_uppercase();

    matches!(
        stem.as_str(),
        "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$"
    ) || (stem.len() == 4
        && matches!(&stem.as_bytes()[..3], b"COM" | b"LPT")
        && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

/// create a symlink for a file
/// if the link exists, it skips the link creation
pub fn symlink_file(original: &Path, link: &Path) -> Result<(), Box<dyn Error>> {
    if !original.is_file() {
        return Err("original path is not a file".into());
    }

    if fs::symlink_metadata(link).is_ok() {
        info!(
            "symbolic link already exists, creation is not necessary: {}",
            link.display()
        );
        return Ok(());
    }

    #[cfg(unix)]
    let result = std::os::unix::fs::symlink(original, link);

    #[cfg(windows)]
    let result = std::os::windows::fs::symlink_file(original, link);

    match result {
        Ok(()) => {
            info!(
                "created symbolic link: {} -> {}",
                link.display(),
                original.display()
            );
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            info!(
                "symbolic link already exists, creation is not necessary: {}",
                link.display()
            );
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn expands_a_leading_user_directory() {
        let home_dir = PathBuf::from(env::var_os("HOME").expect("HOME must be defined"));

        assert_eq!(
            expand_user_dir("~/wallpapers/image.png"),
            home_dir.join("wallpapers/image.png")
        );
    }

    #[test]
    fn expands_user_directory_on_its_own() {
        let home_dir = PathBuf::from(env::var_os("HOME").expect("HOME must be defined"));

        assert_eq!(expand_user_dir("~"), home_dir);
    }

    #[test]
    fn leaves_paths_without_a_leading_user_directory_unchanged() {
        assert_eq!(
            expand_user_dir("/var/lib/nenechi"),
            PathBuf::from("/var/lib/nenechi")
        );
    }

    #[test]
    fn path_match_any_pattern_returns_true_for_matching_path() {
        let patterns = [
            "**/wallpapers/*.mp4".to_string(),
            "wallpapers/*.avi".to_string(),
            "wallpapers/**/*.png".to_string(),
            "wallpapers/cats".to_string(),
            "wallpapers/**/my_favorites".to_string(),
            "wallpapers/cities/**/*".to_string(),
        ];

        assert_eq!(
            true,
            path_match_any_pattern(
                Path::new("/Users/megumin/Images/wallpapers/dogs.mp4"),
                &patterns
            )
        );

        assert_eq!(
            true,
            path_match_any_pattern(Path::new("wallpapers/dogs.avi"), &patterns)
        );

        assert_eq!(
            true,
            path_match_any_pattern(Path::new("wallpapers/landscapes/mountain.png"), &patterns)
        );

        assert_eq!(
            true,
            path_match_any_pattern(Path::new("wallpapers/cats"), &patterns)
        );

        assert_eq!(
            true,
            path_match_any_pattern(Path::new("wallpapers/cats/my_favorites"), &patterns)
        );

        assert_eq!(
            true,
            path_match_any_pattern(
                Path::new("wallpapers/cats/my_favorites/white-cat.png"),
                &patterns
            )
        );

        assert_eq!(
            true,
            path_match_any_pattern(Path::new("wallpapers/cities/london/museum"), &patterns)
        );

        assert_eq!(
            true,
            path_match_any_pattern(
                Path::new("wallpapers/cities/london/big-ben.jpeg"),
                &patterns
            )
        );

        assert_eq!(
            true,
            path_match_any_pattern(
                Path::new("wallpapers/cities/london/river/boat.jpeg"),
                &patterns
            )
        );
    }

    #[test]
    fn sanitizes_filename_components_for_windows_and_smb() {
        assert_eq!(
            sanitize_filename_component("Anime: Part 2?"),
            "Anime_ Part 2_"
        );
        assert_eq!(sanitize_filename_component("CON.mkv"), "_CON.mkv");
        assert_eq!(
            sanitize_filename_component("Título 日本語"),
            "Título 日本語"
        );
        assert_eq!(sanitize_filename_component("trailing. "), "trailing");
    }

    #[test]
    fn sanitizes_every_component_of_a_relative_path() {
        assert_eq!(
            sanitize_relative_path(Path::new("Movie: Name/Extras?/CON.srt")),
            PathBuf::from("Movie_ Name/Extras_/_CON.srt")
        );
    }
}
