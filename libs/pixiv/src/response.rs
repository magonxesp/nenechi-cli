use serde::Deserialize;

#[derive(Deserialize)]
pub struct StatusResponse {
    pub error: bool,
    pub message: String,
}

pub fn verify_response_is_not_error(json: &String) -> Result<(), String> {
    let status: StatusResponse = serde_json::from_str(json.as_str())
        .map_err(|e|  e.to_string())?;

    if !status.error {
        return Ok(())
    }

    let mut message = "pixiv responded with unknown error or the illustration is not found".to_string();
    if !status.message.is_empty() {
        message = status.message
    }

    Err(message)
}
