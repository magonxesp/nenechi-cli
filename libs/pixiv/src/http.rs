use crate::client::create_http_client;
use log::debug;

pub fn http_get_json(url: &str) -> Result<String, String> {
    let client = create_http_client();
    let response = client.get(url).send();
    let response = match response {
        Ok(res) => {
            debug!(
                "url {} responded status code: {}",
                url,
                res.status().to_string()
            );
            res
        }
        Err(e) => {
            debug!("failed requesting json for url: {}", url);
            return Err(e.to_string());
        }
    };

    let content = response.text();
    let content = match content {
        Ok(content) => {
            debug!(
                "decoded response body as plain text for url: {}; content: {}",
                url, content
            );
            content
        }
        Err(e) => {
            debug!("failed decoding response body for url: {}", url);
            return Err(e.to_string());
        }
    };

    Ok(content)
}
