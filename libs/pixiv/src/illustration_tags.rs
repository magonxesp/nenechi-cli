use serde::Deserialize;

use crate::{client::create_http_client, id::IllustrationId};

#[derive(Deserialize, Default, Debug)]
pub struct TagTranslation {
    #[serde(default)]
    pub en: String
}

#[derive(Deserialize, Default, Debug)]
pub struct Tag {
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub romaji: String,
    #[serde(default)]
    pub translation: TagTranslation,
}

#[derive(Deserialize, Default)]
struct Tags {
    #[serde(default)]
    tags: Vec<Tag>
}

#[derive(Deserialize)]
struct TagsConfig {
    tags: Tags
}

#[derive(Deserialize)]
struct Response {
    body: TagsConfig
}

pub fn fetch_tags(id: &IllustrationId) -> Result<Vec<Tag>, String> {
    let url = format!(
        "https://www.pixiv.net/ajax/illust/{id}?lang=en",
        id = id.value
    );

    let client = create_http_client();
    let response = client.get(&url)
        .send()
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())?;

    let response: Response = serde_json::from_str(response.as_str())
        .map_err(|e|  e.to_string())?;

    Ok(response.body.tags.tags)
}
