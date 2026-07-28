use crate::config::DatabaseConfig;
#[cfg(not(test))]
use crate::config::read_config;
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use log::{info, warn};
use std::fs;
use std::sync::OnceLock;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub type DatabasePool = Pool<ConnectionManager<SqliteConnection>>;

static DATABASE_CONNECTION: OnceLock<DatabasePool> = OnceLock::new();

pub fn get_database_connection() -> &'static DatabasePool {
    DATABASE_CONNECTION.get_or_init(|| {
        #[cfg(not(test))]
        let config = read_config().database;
        #[cfg(test)]
        let config = DatabaseConfig::test();

        create_db_connection(&config)
    })
}

fn create_db_connection(config: &DatabaseConfig) -> DatabasePool {
    let path = config.path();
    if let None = &path {
        warn!("database path cannot be resolved: {}", config.file);
        panic!("database path cannot be resolved")
    }

    if let Some(directory) = config.directory()
        && !directory.exists()
    {
        info!("creating database directory at {:?}", directory);
        if let Err(err) = fs::create_dir_all(&directory) {
            warn!("failed to create directory '{:?}': {}", directory, err);
            panic!("failed to create directory for store database");
        }
    }

    let path = path.unwrap();
    if !path.exists() {
        info!("creating database at {:?}", path);
        if let Err(err) = fs::File::create(&path) {
            warn!("failed to create database file '{:?}': {}", path, err);
            panic!("failed to create database file");
        }
    }

    let manager = ConnectionManager::<SqliteConnection>::new(config.sqlite_uri());
    let pool = Pool::builder().build(manager).expect("Error creating pool");

    let mut connection = pool.get().expect("Error connecting to database");

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Error running migrations");

    pool
}

#[cfg(test)]
pub mod tests {
    use crate::database::{DatabasePool, get_database_connection};
    use crate::schema::{series_metadata, wallpapers};
    use diesel::RunQueryDsl;

    #[test]
    fn database_connection_is_a_singleton() {
        let first = get_database_connection();
        let second = get_database_connection();

        assert!(std::ptr::eq(first, second));
    }

    pub fn test_db_connection() -> &'static DatabasePool {
        let connection_pool = get_database_connection();
        let mut connection = connection_pool.get().unwrap();
        diesel::delete(wallpapers::table)
            .execute(&mut connection)
            .unwrap();
        diesel::delete(series_metadata::table)
            .execute(&mut connection)
            .unwrap();
        connection_pool
    }
}
