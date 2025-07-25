use regex::Regex;

pub struct IllustrationId {
    pub value: String
}

impl IllustrationId {
    pub fn new(id: &str) -> Result<Self, String> {
        if !Self::is_valid_id(id) {
            return Err("the illustration id should be numeric".to_string());
        }

        Ok(Self { value: id.to_string() })
    }

    fn is_valid_id(id: &str) -> bool {
        let regex = Regex::new(r"[0-9]+").unwrap();
        regex.is_match(id)
    }
}
