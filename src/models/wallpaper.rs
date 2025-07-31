use std::path::Path;
use diesel::associations::HasTable;
use diesel::prelude::*;
use diesel::result::Error;
use log::warn;
use nenechi_image::AspectRatio;
use crate::schema::wallpapers;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wallpaper {
    pub id: String,
    pub pixiv_illustration_id: Option<String>,
    pub tags: Vec<String>,
    pub aspect_ratio: AspectRatio,
    pub path: String,
    pub file_name: String,
}

impl Wallpaper {
    fn update_from_other(&mut self, model: &Wallpaper) {
        self.pixiv_illustration_id = model.pixiv_illustration_id.clone();
        self.tags = model.tags.clone();
        self.aspect_ratio = model.aspect_ratio.clone();
        self.path = model.path.clone();
        self.file_name = model.file_name.clone();
    }
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

pub struct WallpaperRepository<'a> {
    connection: &'a mut SqliteConnection,
}

impl<'a> WallpaperRepository<'a> {
    pub fn new(connection: &'a mut SqliteConnection) -> Self {
        Self { connection }
    }

    pub fn find_by_id(&self, id: &str) -> Result<Option<Wallpaper>, Box<dyn std::error::Error>> {
        let result: Option<WallpaperTable> = wallpapers::table
            .find(id)
            .first(self.connection)
            .optional()?;

        match result {
            Some(table_model) => Ok(Some(table_model.try_into()?)),
            None => Ok(None)
        }
    }

    pub fn save(&self, wallpaper: Wallpaper) -> Result<(), Box<dyn std::error::Error>> {
        let table_model: WallpaperTable = wallpaper.clone().try_into()?;
        let existing = self.find_by_id(&wallpaper.id)?;

        if existing.is_some() {
            self.update(&table_model)?;
        } else {
            self.insert(&table_model)?;
        }

        Ok(())
    }

    fn insert(&self, table_model: &WallpaperTable) -> Result<(), Box<dyn std::error::Error>> {
        diesel::insert_into(wallpapers::table)
            .values(table_model)
            .execute(self.connection)?;
        Ok(())
    }

    fn update(&self, table_model: &WallpaperTable) -> Result<(), Box<dyn std::error::Error>> {
        diesel::update(wallpapers::table)
            .filter(wallpapers::id.eq(&table_model.id))
            .set((
                wallpapers::pixiv_illustration_id.eq(&table_model.pixiv_illustration_id),
                wallpapers::tags.eq(&table_model.tags),
                wallpapers::aspect_ratio.eq(&table_model.aspect_ratio),
                wallpapers::path.eq(&table_model.path),
                wallpapers::file_name.eq(&table_model.file_name),
            ))
            .execute(self.connection)?;
        Ok(())
    }
}