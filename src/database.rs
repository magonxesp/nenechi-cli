use std::fs;
use std::path::Path;
use crate::config::DatabaseConfig;
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use log::{info, warn};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn create_db_connection(config: &DatabaseConfig) -> Pool<ConnectionManager<SqliteConnection>> {
    let path = config.path();
    if let None = &path {
        warn!("database path cannot be resolved: {}", config.file);
        panic!("database path cannot be resolved")
    }

    if let Some(directory) = config.directory() && !directory.exists() {
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
    let pool = Pool::builder()
        .build(manager)
        .expect("Error creating pool");

    let mut connection = pool.get().expect("Error connecting to database");

    connection.run_pending_migrations(MIGRATIONS)
        .expect("Error running migrations");

    pool
}

#[cfg(test)]
pub mod tests {
    use crate::config::DatabaseConfig;
    use crate::database::create_db_connection;
    use diesel::SqliteConnection;
    use diesel::r2d2::{ConnectionManager, Pool};

    pub fn test_db_connection() -> Pool<ConnectionManager<SqliteConnection>> {
        let config = DatabaseConfig::test();
        config.delete_database_file();
        create_db_connection(&config)
    }
}
