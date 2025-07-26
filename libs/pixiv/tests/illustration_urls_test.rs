use nenechi_pixiv::{fetch_image_urls, IllustrationId};

#[test]
fn it_should_fetch_image_urls_by_id() {
    let id = IllustrationId::new("110177583").unwrap();
    let urls = fetch_image_urls(&id).unwrap();

    dbg!(&urls);
    assert_eq!("https://i.pximg.net/c/48x48/img-master/img/2023/07/23/21/59/38/110177583_p0_square1200.jpg", urls.mini);
    assert_eq!("https://i.pximg.net/c/250x250_80_a2/img-master/img/2023/07/23/21/59/38/110177583_p0_square1200.jpg", urls.thumb);
    assert_eq!("https://i.pximg.net/c/540x540_70/img-master/img/2023/07/23/21/59/38/110177583_p0_master1200.jpg", urls.small);
    assert_eq!("https://i.pximg.net/img-master/img/2023/07/23/21/59/38/110177583_p0_master1200.jpg", urls.regular);
    assert_eq!("https://i.pximg.net/img-original/img/2023/07/23/21/59/38/110177583_p0.png", urls.original);
}
