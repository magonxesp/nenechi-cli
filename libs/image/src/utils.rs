use std::path::Path;
use image::ImageFormat;

pub fn is_image_file(path: &Path) -> bool {
    match ImageFormat::from_path(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}
