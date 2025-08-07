use serde::Deserialize;
use log::debug;
use crate::id::IllustrationId;
use crate::http::http_get_json;
use crate::response::verify_response_is_not_error;

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

    debug!("requesting tags for illustration with id {}: {}", id.value, url);

    let json = http_get_json(url.as_str())?;
    verify_response_is_not_error(&json)?;

    let response: Response = serde_json::from_str(json.as_str())
        .map_err(|e|  e.to_string())?;

    Ok(response.body.tags.tags)
}
