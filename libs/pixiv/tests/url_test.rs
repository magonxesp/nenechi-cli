use pixiv::PixivUrl;

#[test]
fn it_should_create_valid_url() {
    let expected = "https://www.pixiv.net/en/artworks/110177583";
    let url = PixivUrl::new(expected).unwrap();

    assert_eq!(expected, url.value);
}

#[test]
#[should_panic(expected = "the url is not a valid pixiv url")]
fn it_should_not_create_invalid_url() {
    let expected = "https://www.google.com";
    PixivUrl::new(expected).unwrap();
}
