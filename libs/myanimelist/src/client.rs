use log::{debug, info, warn};
use reqwest::Method;
use reqwest::blocking::{RequestBuilder, Response};
use reqwest::header::{HeaderValue, InvalidHeaderValue};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::ErrorKind;

const API_BASE_URL: &str = "https://api.myanimelist.net/v2";
const CLIENT_ID_HEADER: &str = "X-MAL-CLIENT-ID";
const MAX_RETRIES: u64 = 3;

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

    /// Envia la peticion con reintentos en caso de que falle.
    /// Esta funcion no se debe llamar en peticiones que envien datos via streaming.
    pub fn try_send(&self, request: RequestBuilder) -> Result<Response, ClientError> {
        let mut retries = 0;

        loop {
            if retries > 0 {
                let seconds = retries * 3;
                info!("waiting {} seconds before retrying", seconds);
                std::thread::sleep(std::time::Duration::from_secs(retries * 3));
            }

            retries += 1;

            let request_clone = match request.try_clone() {
                Some(req) => req,
                None => return Err(ClientError::RequestNotCloneable),
            };

            let response = request_clone.send();

            if retries > MAX_RETRIES {
                return response.map_err(ClientError::Http);
            }

            if let Err(error) = &response {
                warn!(
                    "reintentando peticion a MyAnimeList por un error en el cliente: {}",
                    error
                );
                continue;
            }

            if let Ok(response) = &response
                && (response.status().is_server_error() || response.status().is_client_error())
            {
                warn!(
                    "reintentando peticion a MyAnimeList porque ha respondido con status: {}",
                    response.status()
                );
                continue;
            }

            return Ok(response.map_err(ClientError::Http)?);
        }
    }
}

#[derive(Debug)]
pub enum ClientError {
    EmptyApiKey,
    EmptySearchQuery,
    RequestNotCloneable,
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
            Self::RequestNotCloneable => write!(formatter, "request not cloneable"),
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
            Self::RequestNotCloneable => None,
            Self::InvalidApiKey(error) => Some(error),
            Self::Http(error) => Some(error),
        }
    }
}
