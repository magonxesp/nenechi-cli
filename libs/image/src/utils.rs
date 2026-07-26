use image::ImageFormat;
use std::path::Path;

pub fn is_image_file(path: &Path) -> bool {
    match ImageFormat::from_path(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}
