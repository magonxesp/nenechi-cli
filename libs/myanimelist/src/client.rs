use log::{debug, info, warn};
use reqwest::{Method, StatusCode};
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

        let api_key = HeaderValue::from_str(api_key.as_ref())
                .map_err(|err| ClientError::InvalidApiKey(err.to_string()))?;

        let http = reqwest::blocking::Client::builder()
            .build()
            .map_err(|err| ClientError::Other(err.to_string()))?;

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
                return response.map_err(|err| ClientError::Http(err.to_string()));
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

            return Ok(response.map_err(|err| ClientError::Http(err.to_string()))?);
        }
    }

    /// comprueba si la peticion ha terminado satisfactoriamente, de lo contrario devuelve el
    /// error correspondiente
    pub fn check_response_error(&self, response: &Response) -> Result<(), ClientError> {
        let status = response.status();

        match status {
            StatusCode::FORBIDDEN => Err(ClientError::InvalidApiKey(format!(
                "{} (expired access tokens or invalid access tokens)", status
            ))),
            StatusCode::BAD_REQUEST => Err(ClientError::Http(format!(
                "{} (invalid parameters)", status
            ))),
            _ => {
                if status.is_server_error() {
                    Err(ClientError::Http(status.to_string()))
                } else if status.is_client_error() {
                    Err(ClientError::Http(status.to_string()))
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ClientError {
    EmptyApiKey,
    EmptySearchQuery,
    RequestNotCloneable,
    InvalidApiKey(String),
    Http(String),
    Other(String)
}

impl Display for ClientError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyApiKey => formatter.write_str("la API key no puede estar vacía"),
            Self::EmptySearchQuery => {
                formatter.write_str("el nombre del anime no puede estar vacío")
            }
            Self::RequestNotCloneable => write!(formatter, "request not cloneable"),
            Self::InvalidApiKey(message) => write!(formatter, "API key no válida: {message}"),
            Self::Http(message) => write!(formatter, "la peticion HTTP ha respondido con status: {message}"),
            Self::Other(message) => write!(formatter, "error en el cliente HTTP: {message}"),
        }
    }
}

impl Error for ClientError {}
