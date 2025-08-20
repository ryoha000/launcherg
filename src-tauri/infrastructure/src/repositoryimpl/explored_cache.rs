use sqlx::QueryBuilder;

use crate::domain::{
    explored_cache::ExploredCache, repository::explored_cache::ExploredCacheRepository,
};

use super::repository::RepositoryImpl;

impl ExploredCacheRepository for RepositoryImpl<ExploredCache> {
    async fn get_all(&self) -> anyhow::Result<ExploredCache> {
        let pool = self.pool.0.clone();
        let paths: Vec<(String,)> = sqlx::query_as("SELECT path from explored_caches")
            .fetch_all(&*pool)
            .await?;
        Ok(paths.into_iter().map(|v| v.0).collect())
    }
    async fn add(&self, cache: ExploredCache) -> anyhow::Result<()> {
        if cache.len() == 0 {
            return Ok(());
        }
        // ref: https://docs.rs/sqlx-core/latest/sqlx_core/query_builder/struct.QueryBuilder.html#method.push_values
        let mut query_builder = QueryBuilder::new("INSERT INTO explored_caches (path) ");
        query_builder.push_values(cache, |mut b, new| {
            b.push_bind(new);
        });

        let pool = self.pool.0.clone();
        let query = query_builder.build();
        query.execute(&*pool).await?;
        Ok(())
    }
}
