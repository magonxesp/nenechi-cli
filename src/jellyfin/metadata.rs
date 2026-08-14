use crate::media::{Actor, Image, SeriesMetadata};
use serde::{Deserialize, Serialize};
use std::path::Path;
use log::debug;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "tvshow")]
pub struct SeriesNfo {
    title: String,
    #[serde(rename = "originaltitle")]
    original_title: String,
    plot: String,
    year: u16,
    premiered: String,
    rating: f32,
    runtime: u16,
    status: String,
    genre: Vec<String>,
    tag: Vec<String>,
    studio: String,
    actor: Vec<Actor>,
    season: u16,
    imdbid: Option<String>,
    tmdbid: Option<String>,
}

impl From<SeriesMetadata> for SeriesNfo {
    fn from(series: SeriesMetadata) -> Self {
        Self {
            title: series.season_title.unwrap_or(series.title),
            original_title: series.season_original_title.unwrap_or(series.original_title),
            plot: series.plot,
            year: series.year,
            premiered: series.premiered,
            rating: series.rating,
            runtime: series.runtime,
            status: series.status,
            genre: series.genre,
            tag: series.tag,
            studio: series.studio,
            tmdbid: series.id.tmdb,
            imdbid: series.id.imdb,
            actor: series.actor,
            season: series.season,
        }
    }
}

impl SeriesNfo {
    pub fn write(&self, path: &Path, season: bool) -> Result<(), String> {
        let filename = match season {
            true => "season.nfo",
            false => "tvshow.nfo",
        };

        let path = path.join(filename);
        let mut xml = String::new();
        xml.insert_str(0, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

        let mut serializer = quick_xml::se::Serializer::new(&mut xml);
        serializer.indent(' ', 4);
        self.serialize(serializer).map_err(|e| format!("error serializing nfo: {}", e))?;

        debug!("writing nfo file: {:?}", path);
        std::fs::write(path, xml).map_err(|e| format!("error writing nfo: {}", e))?;

        Ok(())
    }
}

pub fn write_poster(path: &Path, image: &Image) -> Result<(), String> {
    let extension = match image.content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/bmp" => "bmp",
        "image/tiff" => "tiff",
        "image/svg+xml" => "svg",
        _ => return Err(format!(
            "unsupported image content type: {}",
            image.content_type
        ))
    };

    let path = path.join(format!("poster.{}", extension));
    debug!("writing poster file: {:?}", path);
    std::fs::write(path, &image.content).map_err(|error| format!("error writing poster: {}", error))
}
