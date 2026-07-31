use crate::fs::{sanitize_filename_component, sanitize_relative_path, symlink_file};
use crate::jellyfin::config::{TargetConfig, TargetType};
use log::warn;
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
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warn!(
                    "failed reading a movie item in {}: {error}",
                    target.source.display()
                );
                continue;
            }
        };
        let path = entry.path();

        if path.is_symlink() || target.ignores(&path) {
            continue;
        }
        if path.is_file() {
            if target.includes(&path) {
                match organize_standalone_movie(target, &path) {
                    Ok(()) => links += 1,
                    Err(error) => warn!("failed organizing movie item {}: {error}", path.display()),
                }
            }
            continue;
        }
        if path.is_dir() {
            match organize_movie_directory(target, &path) {
                Ok(directory_links) => links += directory_links,
                Err(error) => warn!(
                    "failed organizing movie directory {}: {error}",
                    path.display()
                ),
            }
        }
    }

    Ok(links)
}

fn organize_standalone_movie(target: &TargetConfig, path: &Path) -> Result<(), Box<dyn Error>> {
    let title = sanitize_filename_component(&file_stem(path)?.to_string_lossy());
    let file_name = sanitize_filename_component(&file_name(path)?.to_string_lossy());
    let destination = target.destination.join(title).join(file_name);
    create_link(path, &destination)
}

fn organize_movie_directory(
    target: &TargetConfig,
    movie_directory: &Path,
) -> Result<usize, Box<dyn Error>> {
    let movie_name = sanitize_filename_component(&file_name(movie_directory)?.to_string_lossy());
    let mut links = 0;

    for entry in WalkDir::new(movie_directory)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !target.ignores(entry.path()))
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warn!(
                    "failed reading an item in movie directory {}: {error}",
                    movie_directory.display()
                );
                continue;
            }
        };
        if !entry.file_type().is_file() || !target.includes(entry.path()) {
            continue;
        }

        let result = (|| {
            let relative = sanitize_relative_path(entry.path().strip_prefix(movie_directory)?);
            let destination = target.destination.join(&movie_name).join(relative);
            create_link(entry.path(), &destination)
        })();
        match result {
            Ok(()) => links += 1,
            Err(error) => warn!(
                "failed organizing movie item {}: {error}",
                entry.path().display()
            ),
        }
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
        let movie = source.join("Example: Movie?.mkv");
        fs::write(&movie, b"movie").unwrap();

        let target = TargetConfig {
            name: "movies".into(),
            target_type: TargetType::Movies,
            source,
            destination: destination.clone(),
            series: None,
            include: vec!["*.mkv".into()],
            ignore: Vec::new(),
        };

        assert_eq!(organize(&target).unwrap(), 1);
        let link = destination
            .join("Example_ Movie_")
            .join("Example_ Movie_.mkv");
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

    #[test]
    fn continues_with_the_next_movie_when_one_fails() {
        let temporary = tempdir().unwrap();
        let source = temporary.path().join("source");
        let destination = temporary.path().join("destination");
        fs::create_dir(&source).unwrap();
        fs::create_dir(&destination).unwrap();
        fs::write(source.join("Broken.mkv"), b"broken").unwrap();
        let working_movie = source.join("Working.mkv");
        fs::write(&working_movie, b"working").unwrap();

        // This file prevents creation of the directory for Broken.mkv.
        fs::write(destination.join("Broken"), b"blocking file").unwrap();

        let target = TargetConfig {
            name: "movies".into(),
            target_type: TargetType::Movies,
            source,
            destination: destination.clone(),
            series: None,
            include: vec!["*.mkv".into()],
            ignore: Vec::new(),
        };

        assert_eq!(organize(&target).unwrap(), 1);
        assert_eq!(
            fs::read_link(destination.join("Working").join("Working.mkv")).unwrap(),
            fs::canonicalize(working_movie).unwrap()
        );
    }
}
