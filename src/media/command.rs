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

        /// Write the resolved media metadata in the current working directory
        /// or the directory path specified by --directory
        #[arg(short, long)]
        write: bool,

        /// Scan for child directories from current working directory
        ///  or the directory path specified by --directory
        #[arg(short, long)]
        scan: bool,

        /// Force writing specific anime metadata from MyAnimeList id
        /// It won't work if you add --scan option
        #[arg(short, long)]
        mal_id: Option<String>,
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
        MediaCommands::Metadata { directory, write, scan, mal_id } => {
            let directory = directory.clone().map(|path| PathBuf::from(path));

            if *scan {
                metadata_recursively(&directory, write)
            } else {
                metadata(&directory, write, mal_id)
            }
        },
    }
}

pub fn metadata(directory: &Option<PathBuf>, write: &bool, media_id: &Option<String>) -> Result<(), String> {
    let path = match directory {
        Some(directory) => directory,
        None => &std::env::current_dir()
            .map_err(|err| format!("failed to get current directory: {}", err))?,
    };

    let resolver = AnimeResolver::build().map_err(|err| err.to_string())?;
    let metadata = if let Some(id) = media_id {
        resolver.resolve_from_identifier(&id).map_err(|err| err.to_string())?
    } else {
        resolver.resolve_from_directory(&path).map_err(|err| err.to_string())?
    };

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

        if let Err(err) = metadata(&Some(path.clone()), write, &None) {
            warn!("failed scanning directory {:?}: {}", path, err);
            continue;
        }
    }

    Ok(())
}
