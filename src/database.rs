use crate::config::DatabaseConfig;
use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn create_db_connection(config: &DatabaseConfig) -> Pool<ConnectionManager<SqliteConnection>> {
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
