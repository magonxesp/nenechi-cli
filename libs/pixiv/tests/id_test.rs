use nenechi_pixiv::IllustrationId;
use std::path::Path;

#[test]
fn it_should_create_valid_illustration_id() {
    let expected = "110177583";
    let id = IllustrationId::new(expected).unwrap();

    assert_eq!(expected, id.value);
}

#[test]
#[should_panic(expected = "the illustration id should be numeric")]
fn it_should_not_create_invalid_illustration_id() {
    let expected = "www.google.com";
    IllustrationId::new(expected).unwrap();
}

#[test]
fn it_should_create_valid_illustration_id_from_path() {
    let id = IllustrationId::from_path(Path::new("resources/89839830_p0.jpg")).unwrap();

    assert_eq!("89839830", id.value);
}

#[test]
fn it_should_create_valid_illustration_id_from_path_with_size() {
    let square_id =
        IllustrationId::from_path(Path::new("resources/96912590_p0_square1200.jpg")).unwrap();
    let master_id =
        IllustrationId::from_path(Path::new("resources/96912590_p0_master1200.jpg")).unwrap();

    assert_eq!("96912590", square_id.value);
    assert_eq!("96912590", master_id.value);
}
