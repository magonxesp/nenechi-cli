use std::io;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Actor {
    pub name: String,
    pub role: String
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetadataProviderIds {
    pub mal: Option<String>,
    pub imdb: Option<String>,
    pub tmdb: Option<String>
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Image {
    #[serde_as(as = "Base64")]
    pub content: Vec<u8>,
    pub content_type: String,
}

pub fn fetch_image_from_url(url: &str) -> io::Result<Image> {
    let client = reqwest::blocking::Client::new();

    let response = client.get(url)
        .send()
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

    if !response.status().is_success() {
        return Err(io::Error::new(io::ErrorKind::Other, format!("HTTP status: {}", response.status())));
    }

    let content_type = response.headers()
        .get("Content-Type")
        .map(|h| h.to_str());

    if content_type.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Response did not contain a Content-Type header"
        ));
    }

    let content_type = match content_type.unwrap() {
        Ok(content_type) => content_type.to_string(),
        Err(error) => return Err(io::Error::new(io::ErrorKind::Other, error)),
    };

    let bytes = response.bytes().map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;

    Ok(Image {
        content: bytes.into(),
        content_type,
    })
}
