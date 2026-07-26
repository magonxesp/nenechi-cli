use crate::schema::wallpapers;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use nenechi_image::AspectRatio;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wallpaper {
    pub id: String,
    pub pixiv_illustration_id: Option<String>,
    pub tags: Vec<String>,
    pub aspect_ratio: AspectRatio,
    pub path: String,
    pub file_name: String,
}

#[derive(Clone, Queryable, Selectable, Insertable)]
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

pub struct WallpaperRepository {
    connection_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl WallpaperRepository {
    pub fn new(connection_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { connection_pool }
    }

    pub fn find_by_id(&self, id: &str) -> Result<Option<Wallpaper>, Box<dyn std::error::Error>> {
        let mut connection = self.connection_pool.get()?;
        let result: Option<WallpaperTable> = wallpapers::table
            .find(id)
            .first(&mut connection)
            .optional()?;

        match result {
            Some(table_model) => Ok(Some(table_model.try_into()?)),
            None => Ok(None)
        }
    }

    pub fn find_by_path(&self, path: &str) -> Result<Option<Wallpaper>, Box<dyn std::error::Error>> {
        let mut connection = self.connection_pool.get()?;
        let result: Option<WallpaperTable> = wallpapers::table
            .filter(wallpapers::path.eq(path))
            .first(&mut connection)
            .optional()?;

        match result {
            Some(table_model) => Ok(Some(table_model.try_into()?)),
            None => Ok(None)
        }
    }

    pub fn save(&self, wallpaper: &Wallpaper) -> Result<(), Box<dyn std::error::Error>> {
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
        let mut connection = self.connection_pool.get()?;
        diesel::insert_into(wallpapers::table)
            .values(table_model)
            .execute(&mut connection)?;
        Ok(())
    }

    fn update(&self, table_model: &WallpaperTable) -> Result<(), Box<dyn std::error::Error>> {
        let mut connection = self.connection_pool.get()?;
        diesel::update(wallpapers::table)
            .filter(wallpapers::id.eq(&table_model.id))
            .set((
                wallpapers::pixiv_illustration_id.eq(&table_model.pixiv_illustration_id),
                wallpapers::tags.eq(&table_model.tags),
                wallpapers::aspect_ratio.eq(&table_model.aspect_ratio),
                wallpapers::path.eq(&table_model.path),
                wallpapers::file_name.eq(&table_model.file_name),
            ))
            .execute(&mut connection)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::tests::test_db_connection;
    use crate::schema::wallpapers;
    use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
    use nenechi_image::AspectRatio;
    use serial_test::serial;
    use uuid::Uuid;

    impl Wallpaper {
        fn test() -> Self {
            Self {
                id: Uuid::now_v7().to_string(),
                pixiv_illustration_id: None,
                tags: vec!["konosuba".to_string(), "megumin".to_string()],
                path: "wallpapers/megumin.jpeg".to_string(),
                file_name: "megumin.jpeg".to_string(),
                aspect_ratio: AspectRatio::Square
            }
        }
    }

    #[test]
    #[serial]
    fn wallpaper_repository_save_should_insert_new_wallpaper() {
        let connection_pool = test_db_connection();
        let mut connection = connection_pool.get().unwrap();
        let repository = WallpaperRepository::new(connection_pool);
        let wallpaper = Wallpaper::test();

        repository.save(&wallpaper).unwrap();

        let result: Option<WallpaperTable> = wallpapers::table
            .find(&wallpaper.id)
            .first(&mut connection)
            .optional()
            .unwrap();
        let existing = result.unwrap()
            .try_into()
            .unwrap();

        assert_eq!(wallpaper, existing);
    }

    #[test]
    #[serial]
    fn wallpaper_repository_save_should_update_existing_wallpaper() {
        let connection_pool = test_db_connection();
        let mut connection = connection_pool.get().unwrap();
        let repository = WallpaperRepository::new(connection_pool);
        let mut wallpaper = Wallpaper::test();

        repository.insert(
            &wallpaper.clone()
                .try_into()
                .unwrap()
        ).unwrap();

        wallpaper.path = "wallpapers/aqua.jpeg".to_string();
        wallpaper.file_name = "aqua.jpeg".to_string();

        repository.save(&wallpaper).unwrap();

        let result: Option<WallpaperTable> = wallpapers::table
            .find(&wallpaper.id)
            .first(&mut connection)
            .optional()
            .unwrap();
        let existing = result.unwrap()
            .try_into()
            .unwrap();

        assert_eq!(wallpaper, existing);
    }

    #[test]
    #[serial]
    fn wallpaper_repository_find_by_id_should_find_existing() {
        let connection_pool = test_db_connection();
        let repository = WallpaperRepository::new(connection_pool);
        let wallpaper = Wallpaper::test();

        repository.save(&wallpaper).unwrap();
        let existing = repository.find_by_id(&wallpaper.id).unwrap();

        assert_eq!(wallpaper, existing.unwrap());
    }

    #[test]
    #[serial]
    fn wallpaper_repository_find_by_id_should_not_find_not_existing() {
        let connection_pool = test_db_connection();
        let repository = WallpaperRepository::new(connection_pool);

        let existing = repository.find_by_id("not_exists").unwrap();

        assert_eq!(None, existing);
    }

    #[test]
    #[serial]
    fn wallpaper_repository_find_by_path_should_find_existing() {
        let connection_pool = test_db_connection();
        let repository = WallpaperRepository::new(connection_pool);
        let wallpaper = Wallpaper::test();

        repository.save(&wallpaper).unwrap();
        let existing = repository.find_by_path(&wallpaper.path).unwrap();

        assert_eq!(wallpaper, existing.unwrap());
    }

    #[test]
    #[serial]
    fn wallpaper_repository_find_by_path_should_not_find_not_existing() {
        let connection_pool = test_db_connection();
        let repository = WallpaperRepository::new(connection_pool);

        let existing = repository.find_by_path("path/not/exists").unwrap();

        assert_eq!(None, existing);
    }
}
