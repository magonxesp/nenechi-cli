use crate::config::DatabaseConfig;
use diesel::{Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn create_db_connection(config: &DatabaseConfig) -> SqliteConnection {
    let mut connection = SqliteConnection::establish(&config.sqlite_uri())
        .unwrap_or_else(|_| panic!("Error connecting to {}", config.sqlite_uri()));

    connection.run_pending_migrations(MIGRATIONS)
        .expect("Error running migrations");

    connection
}

#[cfg(test)]
pub mod tests {
    use crate::config::DatabaseConfig;
    use crate::database::create_db_connection;
    use diesel::SqliteConnection;

    pub fn test_db_connection() -> SqliteConnection {
        let config = DatabaseConfig::test();
        config.delete_database_file();
        create_db_connection(&config)
    }
}
