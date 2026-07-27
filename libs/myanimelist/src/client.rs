use std::error::Error;
use std::fmt::{self, Display, Formatter};

use reqwest::Method;
use reqwest::blocking::RequestBuilder;
use reqwest::header::{HeaderValue, InvalidHeaderValue};

const API_BASE_URL: &str = "https://api.myanimelist.net/v2";
const CLIENT_ID_HEADER: &str = "X-MAL-CLIENT-ID";

#[derive(Clone)]
pub struct Client {
    http: reqwest::blocking::Client,
    api_key: HeaderValue,
}

impl Client {
    pub fn new(api_key: impl AsRef<str>) -> Result<Self, ClientError> {
        if api_key.as_ref().is_empty() {
            return Err(ClientError::EmptyApiKey);
        }

        let api_key =
            HeaderValue::from_str(api_key.as_ref()).map_err(ClientError::InvalidApiKey)?;

        let http = reqwest::blocking::Client::builder()
            .no_proxy()
            .build()
            .map_err(ClientError::Http)?;

        Ok(Self { http, api_key })
    }

    pub fn get(&self, path: &str) -> RequestBuilder {
        self.request(Method::GET, path)
    }

    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{API_BASE_URL}/{}", path.trim_start_matches('/'));
        self.http
            .request(method, url)
            .header(CLIENT_ID_HEADER, self.api_key.clone())
    }
}

#[derive(Debug)]
pub enum ClientError {
    EmptyApiKey,
    EmptySearchQuery,
    InvalidApiKey(InvalidHeaderValue),
    Http(reqwest::Error),
}

impl Display for ClientError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyApiKey => formatter.write_str("la API key no puede estar vacía"),
            Self::EmptySearchQuery => {
                formatter.write_str("el nombre del anime no puede estar vacío")
            }
            Self::InvalidApiKey(error) => write!(formatter, "API key no válida: {error}"),
            Self::Http(error) => write!(formatter, "no se pudo crear el cliente HTTP: {error}"),
        }
    }
}

impl Error for ClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EmptyApiKey => None,
            Self::EmptySearchQuery => None,
            Self::InvalidApiKey(error) => Some(error),
            Self::Http(error) => Some(error),
        }
    }
}
