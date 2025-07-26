use nenechi_image::ImageDetails;

#[test]
fn it_should_read_image_metadata() {
    let details = ImageDetails::read_from_path("./resources/megumin.jpg").unwrap();

    assert_eq!(640, details.width);
    assert_eq!(640, details.height);
}
