use std::path::Path;
use glob::{MatchOptions, Pattern};
use log::warn;

pub fn path_match_any_pattern(path: &Path, patterns: Vec<String>) -> bool {
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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn path_match_any_pattern_returns_true_for_matching_path() {
        let patterns = vec![
            "**/wallpapers/*.mp4".to_string(),
            "wallpapers/*.avi".to_string(),
            "wallpapers/**/*.png".to_string(),
            "wallpapers/cats".to_string(),
            "wallpapers/**/my_favorites".to_string(),
            "wallpapers/cities/**/*".to_string(),
        ];

        assert_eq!(true, path_match_any_pattern(
            Path::new("/Users/megumin/Images/wallpapers/dogs.mp4"),
            patterns.clone()
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/dogs.avi"),
            patterns.clone()
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/landscapes/mountain.png"),
            patterns.clone()
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cats"),
            patterns.clone()
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cats/my_favorites"),
            patterns.clone()
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cats/my_favorites/white-cat.png"),
            patterns.clone()
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cities/london/museum"),
            patterns.clone()
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cities/london/big-ben.jpeg"),
            patterns.clone()
        ));

        assert_eq!(true, path_match_any_pattern(
            Path::new("wallpapers/cities/london/river/boat.jpeg"),
            patterns.clone()
        ));
    }
}
