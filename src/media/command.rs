use crate::media::anime::AnimeResolver;
use crate::media::{series, SeriesMetadataResolver};
use clap::Subcommand;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use log::{info, warn};
use walkdir::WalkDir;

#[derive(Clone, Debug, Subcommand)]
pub enum MediaCommands {
    Metadata {
        #[arg(short, long)]
        directory: Option<String>,

        #[arg(short, long)]
        write: bool,

        #[arg(short, long)]
        scan: bool,
    },
}

impl Display for MediaCommands {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Metadata { .. } => formatter.write_str("metadata"),
        }
    }
}

pub fn execute_media_command(command: &MediaCommands) -> Result<(), String> {
    match command {
        MediaCommands::Metadata { directory, write, scan } => {
            let directory = directory.clone().map(|path| PathBuf::from(path));

            if *scan {
                metadata_recursively(&directory, write)
            } else {
                metadata(&directory, write)
            }
        },
    }
}

pub fn metadata(directory: &Option<PathBuf>, write: &bool) -> Result<(), String> {
    let path = match directory {
        Some(directory) => directory,
        None => &std::env::current_dir()
            .map_err(|err| format!("failed to get current directory: {}", err))?,
    };

    let resolver = AnimeResolver::build().map_err(|err| err.to_string())?;
    let metadata = resolver.resolve(&path).map_err(|err| err.to_string())?;

    if !write {
        println!("{}:\n{}", path.display(), metadata.to_yaml()?);
        return Ok(());
    }

    metadata.write(&path)?;
    info!("wrote metadata to {}", path.display());
    Ok(())
}

pub fn metadata_recursively(directory: &Option<PathBuf>, write: &bool) -> Result<(), String> {
    let path = match directory {
        Some(directory) => directory,
        None => &std::env::current_dir()
            .map_err(|err| format!("failed to get current directory: {}", err))?,
    };

    let child_directories = WalkDir::new(&path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| entry.file_type().is_dir());

    for entry in child_directories {
        let entry = entry.map_err(|err| format!("failed to read directory entry: {}", err))?;
        let path = entry.path().to_path_buf();

        if let Err(err) = metadata(&Some(path.clone()), write) {
            warn!("failed scanning directory {:?}: {}", path, err);
            continue;
        }
    }

    Ok(())
}
