#[derive(Debug)]
pub struct ImageDetails {
    pub width: u32,
    pub height: u32
}

impl ImageDetails {
    pub fn read_from_path(path: &str) -> Result<Self, String> {
        let metadata = image::open(path)
            .map_err(|e| e.to_string())?;

        Ok(Self {
            width: metadata.width(),
            height: metadata.height()
        })
    }

    pub fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    pub fn is_portrait(&self) -> bool {
        self.width < self.height
    }

    pub fn is_square(&self) -> bool {
        self.width == self.height
    }
}
