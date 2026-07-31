use regex::Regex;
use std::path::Path;

#[derive(Clone)]
pub struct IllustrationId {
    pub value: String,
}

impl IllustrationId {
    pub fn new(id: &str) -> Result<Self, String> {
        if !Self::is_valid_id(id) {
            return Err("the illustration id should be numeric".to_string());
        }

        Ok(Self {
            value: id.to_string(),
        })
    }

    pub fn from_path(path: &Path) -> Result<Self, String> {
        if !path.is_file() {
            return Err(format!("path {} is not a file", path.display()));
        }

        let file_name = path
            .file_name()
            .ok_or(format!(
                "unable to resolve the file name for path {}",
                path.display()
            ))?
            .to_str()
            .ok_or(format!(
                "unable to casting to string file name for for path {}",
                path.display()
            ))?;
        let file_name_regex = Regex::new(r"([0-9]+)_.*").unwrap();

        if !file_name_regex.is_match(&file_name) {
            return Err(format!("file name {} is invalid", file_name));
        }

        let error = format!(
            "the illustration id is not present on the file: {}",
            path.display()
        );
        let id = file_name_regex
            .captures(file_name)
            .ok_or(&error)?
            .get(1)
            .ok_or(&error)?
            .as_str();

        Ok(Self::new(id)?)
    }

    fn is_valid_id(id: &str) -> bool {
        let regex = Regex::new(r"[0-9]+").unwrap();
        regex.is_match(id)
    }
}
