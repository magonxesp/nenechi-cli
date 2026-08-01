use crate::database::{DatabasePool, get_database_connection};
use crate::schema::series_metadata;
use diesel::prelude::*;
use std::error::Error;
use std::path::Path;
use std::sync::OnceLock;
use uuid::Uuid;

static SERIES_METADATA_REPOSITORY: OnceLock<SeriesMetadataRepository> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeriesMetadata {
    pub id: String,
    pub title: String,
    pub path: String,
    pub season: i64,
}

impl SeriesMetadata {
    pub fn new(title: String, path: String, season: Option<i64>) -> Result<Self, Box<dyn Error>> {
        let title = title.trim().to_string();
        if title.is_empty() {
            return Err("series title cannot be empty".into());
        }
        if path.trim().is_empty() {
            return Err("series path cannot be empty".into());
        }

        let season = season.unwrap_or(1);
        if season == 0 {
            return Err("series season must be greater than zero".into());
        }

        Ok(Self {
            id: Uuid::now_v7().to_string(),
            title,
            path,
            season,
        })
    }

    pub fn from_directory(path: &Path, season: Option<i64>) -> Result<Self, Box<dyn Error>> {
        let title = path
            .file_name()
            .ok_or_else(|| format!("series path {} has no directory name", path.display()))?
            .to_str()
            .ok_or_else(|| format!("series path {} is not valid UTF-8", path.display()))?
            .to_string();
        let path = path
            .to_str()
            .ok_or_else(|| format!("series path {} is not valid UTF-8", path.display()))?
            .to_string();

        Self::new(title, path, season)
    }
}

#[derive(Clone, Debug, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::series_metadata)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct SeriesMetadataTable {
    id: String,
    title: String,
    path: String,
    season: i32,
}

impl TryFrom<SeriesMetadata> for SeriesMetadataTable {
    type Error = Box<dyn Error>;

    fn try_from(value: SeriesMetadata) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            title: value.title,
            path: value.path,
            season: i32::try_from(value.season)?,
        })
    }
}

impl TryFrom<SeriesMetadataTable> for SeriesMetadata {
    type Error = Box<dyn Error>;

    fn try_from(value: SeriesMetadataTable) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            title: value.title,
            path: value.path,
            season: i64::try_from(value.season)?,
        })
    }
}

pub struct SeriesMetadataRepository {
    connection_pool: DatabasePool,
}

impl SeriesMetadataRepository {
    pub fn new(connection_pool: &DatabasePool) -> Self {
        Self {
            connection_pool: connection_pool.clone(),
        }
    }

    pub fn get_instance() -> &'static Self {
        SERIES_METADATA_REPOSITORY.get_or_init(|| Self::new(get_database_connection()))
    }

    pub fn find_by_path(&self, path: &str) -> Result<Option<SeriesMetadata>, Box<dyn Error>> {
        let mut connection = self.connection_pool.get()?;
        let result = series_metadata::table
            .filter(series_metadata::path.eq(path))
            .first::<SeriesMetadataTable>(&mut connection)
            .optional()?;

        result.map(TryInto::try_into).transpose()
    }

    pub fn save(&self, metadata: &SeriesMetadata) -> Result<(), Box<dyn Error>> {
        let mut connection = self.connection_pool.get()?;
        let table_model: SeriesMetadataTable = metadata.clone().try_into()?;

        diesel::insert_into(series_metadata::table)
            .values(&table_model)
            .on_conflict(series_metadata::path)
            .do_update()
            .set((
                series_metadata::title.eq(&table_model.title),
                series_metadata::season.eq(table_model.season),
            ))
            .execute(&mut connection)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_season_to_one() {
        let metadata =
            SeriesMetadata::new("Example".into(), "/series/Example".into(), None).unwrap();

        assert_eq!(metadata.season, 1);
        assert!(Uuid::parse_str(&metadata.id).is_ok());
    }

    #[test]
    fn rejects_season_zero() {
        assert!(SeriesMetadata::new("Example".into(), "/series/Example".into(), Some(0)).is_err());
    }
}
