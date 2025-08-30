use domain::{explored_cache::ExploredCache, repository::explored_cache::ExploredCacheRepository};
use crate::sqliterepository::sqliterepository::SqliteRepository;

impl<'a> ExploredCacheRepository for SqliteRepository<'a> {
    async fn get_all(&mut self) -> anyhow::Result<ExploredCache> {
        let paths: Vec<(String,)> = self.executor.with_conn(|conn| {
            Box::pin(async move { Ok(sqlx::query_as("SELECT path from explored_caches").fetch_all(conn).await?) })
        }).await?;
        Ok(paths.into_iter().map(|v| v.0).collect())
    }
    async fn add(&mut self, cache: ExploredCache) -> anyhow::Result<()> {
        if cache.is_empty() { return Ok(()); }
        self.executor.with_conn(|conn| {
            Box::pin(async move {
                let mut qb = sqlx::QueryBuilder::new("INSERT INTO explored_caches (path) ");
                qb.push_values(cache, |mut b, new| { b.push_bind(new); });
                qb.build().execute(conn).await?;
                Ok::<(), anyhow::Error>(())
            })
        }).await?;
        Ok(())
    }
}


