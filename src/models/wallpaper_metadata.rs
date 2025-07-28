use diesel::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WallpaperAspectRatio {
    Landscape,
    Portrait,
    Square,
}

impl WallpaperAspectRatio {
    pub fn from_string(value: &String) -> Result<Self, String> {
        match value.as_str() {
            "Landscape" => Ok(Self::Landscape),
            "Portrait" => Ok(Self::Portrait),
            "Square" => Ok(Self::Square),
            _ => Err(format!("Invalid aspect ratio value: {}", value)),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Landscape => String::from("Landscape"),
            Self::Portrait => String::from("Portrait"),
            Self::Square => String::from("Square"),
        }
    }
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::wallpaper_metadata)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct WallpaperMetadata {
    pub id: String,
    pub pixiv_illustration_id: Option<String>,
    tags: String,
    aspect_ratio: String,
}

impl WallpaperMetadata {
    pub fn aspect_ratio(&self) -> Option<WallpaperAspectRatio> {
        match WallpaperAspectRatio::from_string(&self.aspect_ratio) {
            Ok(r) => Some(r),
            Err(_) => None,
        }
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: WallpaperAspectRatio) {
        self.aspect_ratio = aspect_ratio.to_string();
    }
}
