use reqwest::blocking::Client;

const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:141.0) Gecko/20100101 Firefox/141.0";

pub fn create_http_client() -> Client {
    Client::builder()
        .user_agent(DEFAULT_USER_AGENT)
        .build()
        .unwrap()
}
