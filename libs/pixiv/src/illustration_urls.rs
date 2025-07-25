use serde::Deserialize;

use crate::{client::create_http_client, IllustrationId};

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

    let client = create_http_client();
    let response = client.get(&url)
        .send()
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())?;

    let response: Response = serde_json::from_str(response.as_str())
        .map_err(|e|  e.to_string())?;

    Ok(response.body.urls)
}
