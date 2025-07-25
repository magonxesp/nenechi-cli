use pixiv::{fetch_tags, IllustrationId};

#[test]
fn it_should_fetch_tags_by_id() {
    let id = IllustrationId::new("110177583").unwrap();
    let tags = fetch_tags(&id).unwrap();

    dbg!(&tags);
    assert_eq!(4, tags.capacity())
}
