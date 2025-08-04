use serde::Deserialize;

use crate::{client::create_http_client, IllustrationId};
use crate::http::http_get_json;
use crate::response::verify_response_is_not_error;

#[derive(Deserialize, Debug)]
pub struct Urls {
	pub mini:     String,
	pub thumb:    String,
	pub small:    String,
	pub regular:  String,
	pub original: String
}

#[derive(Deserialize)]
struct UrlsConfig {
    urls: Urls
}

#[derive(Deserialize)]
struct Response {
    body: UrlsConfig
}

pub fn fetch_image_urls(id: &IllustrationId) -> Result<Urls, String> {
    let url = format!(
        "https://www.pixiv.net/ajax/illust/{id}?lang=en",
        id = id.value
    );

    let json = http_get_json(url.as_str())?;
    verify_response_is_not_error(&json)?;

    let response: Response = serde_json::from_str(json.as_str())
        .map_err(|e|  e.to_string())?;

    Ok(response.body.urls)
}
