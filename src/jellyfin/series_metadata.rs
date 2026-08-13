use crate::database::{DatabasePool};
use crate::schema::series_metadata;
use diesel::prelude::*;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::num::TryFromIntError;
use std::path::Path;
use std::sync::OnceLock;
use uuid::Uuid;
use crate::config::CliConfig;
use crate::database;

static SERIES_METADATA_REPOSITORY: OnceLock<SeriesMetadataRepository> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeriesMetadata {
    pub id: String,
    pub title: String,
    pub path: String,
    pub season: i64,
}

impl SeriesMetadata {
    pub fn new(title: String, path: String, season: Option<i64>) -> Result<Self, String> {
        let title = title.trim().to_string();
        if title.is_empty() {
            return Err("series title cannot be empty".to_string());
        }
        if path.trim().is_empty() {
            return Err("series path cannot be empty".to_string());
        }

        let season = season.unwrap_or(1);
        if season == 0 {
            return Err("series season must be greater than zero".to_string());
        }

        Ok(Self {
            id: Uuid::now_v7().to_string(),
            title,
            path,
            season,
        })
    }

    pub fn from_directory(path: &Path, season: Option<i64>) -> Result<Self, String> {
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

#[derive(Debug)]
pub enum SeriesMetadataRepositoryError {
    Connection(diesel::r2d2::PoolError),
    Query(diesel::result::Error),
    SeasonOutOfRange(TryFromIntError),
}

impl From<diesel::r2d2::PoolError> for SeriesMetadataRepositoryError {
    fn from(error: diesel::r2d2::PoolError) -> Self {
        Self::Connection(error)
    }
}

impl From<diesel::result::Error> for SeriesMetadataRepositoryError {
    fn from(error: diesel::result::Error) -> Self {
        Self::Query(error)
    }
}

impl From<TryFromIntError> for SeriesMetadataRepositoryError {
    fn from(error: TryFromIntError) -> Self {
        Self::SeasonOutOfRange(error)
    }
}

impl Display for SeriesMetadataRepositoryError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(error) => {
                write!(formatter, "failed to get database connection: {error}")
            }
            Self::Query(error) => write!(formatter, "database query failed: {error}"),
            Self::SeasonOutOfRange(_) => formatter.write_str("series season is out of range"),
        }
    }
}

impl Error for SeriesMetadataRepositoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connection(error) => Some(error),
            Self::Query(error) => Some(error),
            Self::SeasonOutOfRange(error) => Some(error),
        }
    }
}

impl TryFrom<SeriesMetadata> for SeriesMetadataTable {
    type Error = SeriesMetadataRepositoryError;

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
    type Error = SeriesMetadataRepositoryError;

    fn try_from(value: SeriesMetadataTable) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            title: value.title,
            path: value.path,
            season: i64::from(value.season),
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

    pub fn from_config(config: &CliConfig) -> Self {
        Self::new(&database::create_db_connection(&config.database))
    }

    pub fn get_instance() -> &'static Self {
        SERIES_METADATA_REPOSITORY.get_or_init(|| Self::new(database::get_database_connection()))
    }

    pub fn find_by_path(
        &self,
        path: &str,
    ) -> Result<Option<SeriesMetadata>, SeriesMetadataRepositoryError> {
        let mut connection = self.connection_pool.get()?;
        let result = series_metadata::table
            .filter(series_metadata::path.eq(path))
            .first::<SeriesMetadataTable>(&mut connection)
            .optional()?;

        result.map(TryInto::try_into).transpose()
    }

    pub fn save(&self, metadata: &SeriesMetadata) -> Result<(), SeriesMetadataRepositoryError> {
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

    #[test]
    fn maps_diesel_query_errors_to_repository_errors() {
        let error = SeriesMetadataRepositoryError::from(diesel::result::Error::NotFound);

        assert!(matches!(
            error,
            SeriesMetadataRepositoryError::Query(diesel::result::Error::NotFound)
        ));
    }

    #[test]
    fn rejects_seasons_that_do_not_fit_in_the_database_column() {
        let metadata = SeriesMetadata::new(
            "Example".into(),
            "/series/Example".into(),
            Some(i64::from(i32::MAX) + 1),
        )
        .unwrap();

        assert!(matches!(
            SeriesMetadataTable::try_from(metadata),
            Err(SeriesMetadataRepositoryError::SeasonOutOfRange(_))
        ));
    }
}
