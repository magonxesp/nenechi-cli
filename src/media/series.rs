use crate::jellyfin::config::SeriesCategory;
use crate::media::metadata::{Actor, MetadataProviderIds};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::media::{metadata, Image};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SeriesMetadata {
    pub title: String,
    #[serde(rename = "originaltitle")]
    pub original_title: String,
    pub plot: String,
    pub year: u16,
    pub premiered: String,
    pub rating: f32,
    pub runtime: u16,
    pub status: String,
    pub genre: Vec<String>,
    pub tag: Vec<String>,
    pub studio: String,
    pub id: MetadataProviderIds,
    pub actor: Vec<Actor>,
    pub season: u16,
    pub season_title: Option<String>,
    pub season_original_title: Option<String>,
    pub cover: Option<Image>,
}

#[derive(Debug)]
pub enum SeriesMetadataResolverError {
    EmptyTitle,
    NotFound,
    Other(String)
}

impl Display for SeriesMetadataResolverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SeriesMetadataResolverError::EmptyTitle => write!(f, "Empty title"),
            SeriesMetadataResolverError::NotFound => write!(f, "Not found"),
            SeriesMetadataResolverError::Other(e) => write!(f, "{}", e)
        }
    }
}

impl Error for SeriesMetadataResolverError {}

pub trait SeriesMetadataResolver {
    fn resolve(&self, source_directory: &Path) -> Result<SeriesMetadata, SeriesMetadataResolverError>;
}

pub const SERIES_METADATA_FILENAME: &str = ".metadata.yaml";

/// returns series metadata in YAML format
pub fn resolve_series_metadata_from_path(path: &Path, resolver: &impl SeriesMetadataResolver) -> Result<String, String> {
    let title = path.file_name()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default();

    if title == "" {
        return Err("title is empty".to_string());
    }

    let metadata = resolver.resolve(path)
        .map_err(|err| err.to_string())?;

    let yaml = serde_yaml::to_string(&metadata)
        .map_err(|err| format!("failed serializing metadata: {}", err))?;

    Ok(yaml)
}

pub fn write_series_metadata_for_path(path: &Path, resolver: &impl SeriesMetadataResolver) -> Result<(), String> {
    let yaml = resolve_series_metadata_from_path(path, resolver)?;

    std::fs::write(path.join(SERIES_METADATA_FILENAME), yaml)
        .map_err(|err| format!("failed writing metadata: {}", err))?;

    Ok(())
}

pub fn read_series_metadata_from_path(path: &Path) -> Result<SeriesMetadata, String> {
    let file = path.join(SERIES_METADATA_FILENAME);

    if !file.exists() {
        return Err(format!("{:?} not found", file));
    }

    let file = File::open(file)
        .map_err(|err| format!("failed opening file: {}", err))?;

    let metadata: SeriesMetadata = serde_yaml::from_reader(file)
        .map_err(|err| format!("failed deserializing metadata: {}", err))?;

    Ok(metadata)
}
