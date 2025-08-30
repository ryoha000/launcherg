use refinery::config::{Config, ConfigDbType};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use tempfile::NamedTempFile;

use super::sqliterepository::{RepositoryExecutor, SqliteRepository};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/migrations");
}

mod transaction_test;
mod works_test;

pub struct TestDatabase {
    pub pool: Pool<Sqlite>,
    _temp_file: NamedTempFile,
}

impl TestDatabase {
    pub async fn new() -> anyhow::Result<Self> {
        let temp_file = NamedTempFile::new()?;
        let db_path = temp_file.path().to_str().unwrap();

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(db_path)
                    .create_if_missing(true)
                    .foreign_keys(true),
            )
            .await?;

        let mut conf = Config::new(ConfigDbType::Sqlite).set_db_path(db_path);
        embedded::migrations::runner()
            .set_abort_divergent(false)
            .run(&mut conf)?;

        Ok(Self { pool, _temp_file: temp_file })
    }

    pub fn sqlite_repository(&self) -> SqliteRepository<'_> {
        SqliteRepository::new(RepositoryExecutor::Pool(&self.pool))
    }
}


