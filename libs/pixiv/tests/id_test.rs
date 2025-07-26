use nenechi_pixiv::IllustrationId;

#[test]
fn it_should_create_valid_illustration_id() {
    let expected = "110177583";
    let url = IllustrationId::new(expected).unwrap();

    assert_eq!(expected, url.value);
}

#[test]
#[should_panic(expected = "the illustration id should be numeric")]
fn it_should_not_create_invalid_illustration_id() {
    let expected = "www.google.com";
    IllustrationId::new(expected).unwrap();
}
