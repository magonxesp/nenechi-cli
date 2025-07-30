use std::path::Path;
use diesel::prelude::*;
use nenechi_image::AspectRatio;

#[derive(Debug, PartialEq, Eq)]
pub struct Wallpaper {
    pub id: String,
    pub pixiv_illustration_id: Option<String>,
    pub tags: Vec<String>,
    pub aspect_ratio: AspectRatio,
    pub path: String,
    pub file_name: String,
}

#[derive(Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::wallpapers)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct WallpaperTable {
    id: String,
    pixiv_illustration_id: Option<String>,
    tags: String,
    aspect_ratio: String,
    path: String,
    file_name: String,
}

impl TryFrom<Wallpaper> for WallpaperTable {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: Wallpaper) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            pixiv_illustration_id: value.pixiv_illustration_id,
            tags: serde_json::to_string(&value.tags)?,
            aspect_ratio: value.aspect_ratio.to_string(),
            path: value.path,
            file_name: value.file_name,
        })
    }
}

impl TryFrom<WallpaperTable> for Wallpaper {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: WallpaperTable) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            pixiv_illustration_id: value.pixiv_illustration_id,
            tags: serde_json::from_str(&value.tags)?,
            aspect_ratio: AspectRatio::from_string(&value.aspect_ratio)?,
            path: value.path,
            file_name: value.file_name,
        })
    }
}
