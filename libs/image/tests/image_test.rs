use std::path::Path;
use nenechi_image::{AspectRatio, ImageDetails};

#[test]
fn it_should_read_image_metadata() {
    let path = Path::new("resources/megumin.jpg");
    let details = ImageDetails::read_from_path(path).unwrap();

    assert_eq!(640, details.width);
    assert_eq!(640, details.height);
}

#[test]
fn it_should_resolve_landscape_aspect_ratio() {
    let path = Path::new("resources/kurumi.jpg");
    let details = ImageDetails::read_from_path(path).unwrap();

    assert_eq!(AspectRatio::Landscape, details.aspect_ratio);
}

#[test]
fn it_should_resolve_portrait_aspect_ratio() {
    let path = Path::new("resources/aqua.jpeg");
    let details = ImageDetails::read_from_path(path).unwrap();

    assert_eq!(AspectRatio::Portrait, details.aspect_ratio);
}

#[test]
fn it_should_resolve_square_aspect_ratio() {
    let path = Path::new("resources/megumin.jpg");
    let details = ImageDetails::read_from_path(path).unwrap();

    assert_eq!(AspectRatio::Square, details.aspect_ratio);
}
