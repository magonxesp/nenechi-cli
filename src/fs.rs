use glob::Pattern;
use log::{debug, warn};
use std::env;
use std::error::Error;
use std::ffi::OsStr;
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
            Err(_) => warn!("ignore pattern {} is not valid and will not be applied", pattern),
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
    let path_str = os_str
        .ok_or("os string is none")?
        .to_str();
    let path_string = path_str
        .ok_or("str is none")?
        .to_string();

    Ok(path_string)
}

/// create a symlink for a file
/// if the link exists, it skips the link creation
pub fn symlink_file(original: &Path, link: &Path) -> Result<(), Box<dyn Error>> {
    if !original.is_file() {
        return Err("original path is not a file".into());
    }

    if link.exists() {
        debug!("symlink already exists, skipping: {}", link.display());
        return Ok(());
    }

    #[cfg(unix)]
    std::os::unix::fs::symlink(original, link)?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(original, link)?;

    Ok(())
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

        assert_eq!(true, path_match_any_pattern(
            Path::new("/Users/megumin/Images/wallpapers/dogs.mp4"),
            &patterns
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/dogs.avi"),
            &patterns
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/landscapes/mountain.png"),
            &patterns
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cats"),
            &patterns
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cats/my_favorites"),
            &patterns
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cats/my_favorites/white-cat.png"),
            &patterns
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cities/london/museum"),
            &patterns
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cities/london/big-ben.jpeg"),
            &patterns
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cities/london/river/boat.jpeg"),
            &patterns
        ));
    }
}
