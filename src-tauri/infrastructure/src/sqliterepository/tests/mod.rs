use refinery::config::{Config, ConfigDbType};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use tempfile::NamedTempFile;

use super::sqliterepository::SqliteRepositories;
use std::sync::Arc;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/migrations");
}

mod all_game_cache_test;
mod dmm_work_pack_test;
mod explored_cache_test;
mod native_host_log_test;
mod save_image_queue_test;
mod work_lnk_test;
mod work_omit_test;
mod work_parent_packs_test;
mod works_extra_test;
mod works;

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

        Ok(Self {
            pool,
            _temp_file: temp_file,
        })
    }

    pub fn sqlite_repository(&self) -> SqliteRepositories {
        SqliteRepositories::new_from_pool(Arc::new(self.pool.clone()))
    }
}
