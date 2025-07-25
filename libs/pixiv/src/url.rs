use regex::Regex;

#[derive(Debug, Clone, PartialEq)]
pub struct PixivUrl {
    pub value: String
}

impl PixivUrl {
    pub fn new(url: &str) -> Result<PixivUrl, &'static str> {
        if !Self::is_valid_url(url) {
            return Err("the url is not a valid pixiv url")
        }

        Ok(Self { value: url.to_string() })
    }

    fn is_valid_url(url: &str) -> bool {
        let regex = Regex::new(r".*\.?pixiv\.net").unwrap();
        regex.is_match(url)
    }
}
