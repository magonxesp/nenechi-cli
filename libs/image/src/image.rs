use crate::AspectRatio;
use image::DynamicImage;
use std::path::Path;

#[derive(Debug)]
pub struct ImageDetails {
    pub width: u32,
    pub height: u32,
    pub aspect_ratio: AspectRatio,
}

impl ImageDetails {
    pub fn read_from_path(path: &Path) -> Result<Self, String> {
        let metadata = image::open(path)
            .map_err(|e| e.to_string())?;

        Ok(Self {
            width: metadata.width(),
            height: metadata.height(),
            aspect_ratio: Self::resolve_aspect_ratio(&metadata),
        })
    }

    fn resolve_aspect_ratio(metadata: &DynamicImage) -> AspectRatio {
        let width = metadata.width();
        let height = metadata.height();

        if width > height {
            return AspectRatio::Landscape;
        }

        if width < height {
            return AspectRatio::Portrait;
        }

        AspectRatio::Square
    }
}
