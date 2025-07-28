use std::sync::Arc;

use chrono::{DateTime, Local};
use refinery::config::{Config, ConfigDbType};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    Pool, Sqlite,
};
use tempfile::NamedTempFile;

use crate::{
    domain::{
        all_game_cache::AllGameCache, collection::CollectionElement, explored_cache::ExploredCache,
        Id,
    },
    infrastructure::repositoryimpl::{
        driver::Db,
        repository::{Repositories, RepositoriesExt, RepositoryImpl},
    },
};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./src/migrations");
}

pub struct TestDatabase {
    pub pool: Pool<Sqlite>,
    pub db: Db,
    pub repositories: Repositories,
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

        let db = Db(Arc::new(pool.clone()));
        let repositories = Repositories::new(db.clone());

        Ok(TestDatabase {
            pool,
            db,
            repositories,
            _temp_file: temp_file,
        })
    }

    pub fn collection_repository(&self) -> &RepositoryImpl<CollectionElement> {
        self.repositories.collection_repository()
    }

    pub fn explored_cache_repository(&self) -> &RepositoryImpl<ExploredCache> {
        self.repositories.explored_cache_repository()
    }

    pub fn all_game_cache_repository(&self) -> &RepositoryImpl<AllGameCache> {
        self.repositories.all_game_cache_repository()
    }
}

pub fn create_test_collection_element_id() -> Id<CollectionElement> {
    Id::new(1)
}

pub fn create_test_collection_element_ids(count: usize) -> Vec<Id<CollectionElement>> {
    (1..=count).map(|i| Id::new(i as i32)).collect()
}

pub fn create_test_datetime() -> DateTime<Local> {
    Local::now()
}

pub fn create_test_explored_paths() -> Vec<String> {
    vec![
        "/test/path/1".to_string(),
        "/test/path/2".to_string(),
        "/test/path/3".to_string(),
    ]
}

pub mod all_game_cache_test;
pub mod collection_test;
pub mod explored_cache_test;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_setup() {
        let test_db = TestDatabase::new().await.unwrap();

        // データベースが正常に作成され、接続できることを確認
        let result = sqlx::query("SELECT 1").fetch_one(&test_db.pool).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_repositories_creation() {
        let test_db = TestDatabase::new().await.unwrap();

        // 各リポジトリが正常にアクセスできることを確認
        let _collection_repo = test_db.collection_repository();
        let _explored_cache_repo = test_db.explored_cache_repository();
        let _all_game_cache_repo = test_db.all_game_cache_repository();
    }
}
