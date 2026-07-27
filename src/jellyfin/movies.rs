use crate::fs::symlink_file;
use crate::jellyfin::config::{TargetConfig, TargetType};
use std::error::Error;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn organize(target: &TargetConfig) -> Result<usize, Box<dyn Error>> {
    if target.target_type != TargetType::Movies {
        return Err(format!("Jellyfin target {:?} is not a movies target", target.name).into());
    }
    target.validate()?;
    target.validate_source()?;
    fs::create_dir_all(&target.destination)?;

    let mut links = 0;
    for entry in fs::read_dir(&target.source)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_symlink() {
            continue;
        }
        if path.is_file() {
            if target.includes(&path) {
                let title = file_stem(&path)?;
                let destination = target.destination.join(title).join(file_name(&path)?);
                create_link(&path, &destination)?;
                links += 1;
            }
            continue;
        }
        if path.is_dir() {
            links += organize_movie_directory(target, &path)?;
        }
    }

    Ok(links)
}

fn organize_movie_directory(
    target: &TargetConfig,
    movie_directory: &Path,
) -> Result<usize, Box<dyn Error>> {
    let movie_name = file_name(movie_directory)?;
    let mut links = 0;

    for entry in WalkDir::new(movie_directory).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() || !target.includes(entry.path()) {
            continue;
        }

        let relative = entry.path().strip_prefix(movie_directory)?;
        let destination = target.destination.join(movie_name).join(relative);
        create_link(entry.path(), &destination)?;
        links += 1;
    }

    Ok(links)
}

fn create_link(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    let parent = destination
        .parent()
        .ok_or_else(|| format!("destination {} has no parent", destination.display()))?;
    fs::create_dir_all(parent)?;
    let source = fs::canonicalize(source)?;
    symlink_file(&source, destination)
}

fn file_name(path: &Path) -> Result<&std::ffi::OsStr, Box<dyn Error>> {
    path.file_name()
        .ok_or_else(|| format!("path {} has no file name", path.display()).into())
}

fn file_stem(path: &Path) -> Result<&std::ffi::OsStr, Box<dyn Error>> {
    path.file_stem()
        .ok_or_else(|| format!("path {} has no file stem", path.display()).into())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::jellyfin::config::TargetConfig;
    use tempfile::tempdir;

    #[test]
    fn organizes_a_standalone_movie_in_its_own_directory() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        fs::create_dir(&source).unwrap();
        let movie = source.join("Example Movie.mkv");
        fs::write(&movie, b"movie").unwrap();

        let target = TargetConfig {
            name: "movies".into(),
            target_type: TargetType::Movies,
            source,
            destination: destination.clone(),
            series: None,
            include: vec!["*.mkv".into()],
        };

        assert_eq!(organize(&target).unwrap(), 1);
        let link = destination.join("Example Movie").join("Example Movie.mkv");
        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(
            fs::read_link(link).unwrap(),
            fs::canonicalize(movie).unwrap()
        );
    }
}
