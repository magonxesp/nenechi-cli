#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WallpaperAspectRatio {
    Landscape,
    Portrait,
    Square,
}

impl WallpaperAspectRatio {
    pub fn from_string(value: &String) -> Result<Self, String> {
        match value.as_str() {
            "landscape" => Ok(Self::Landscape),
            "portrait" => Ok(Self::Portrait),
            "square" => Ok(Self::Square),
            _ => Err(format!("Invalid aspect ratio value: {}", value)),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Landscape => String::from("landscape"),
            Self::Portrait => String::from("portrait"),
            Self::Square => String::from("square"),
        }
    }
}
