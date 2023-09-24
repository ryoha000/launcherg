use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDateTime};
use sqlx::{query_as, QueryBuilder, Row};

use crate::domain::{
    all_game_cache::{AllGameCache, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne},
    repository::all_game_cache::AllGameCacheRepository,
};

use super::{models::all_game_cache::AllGameCacheTable, repository::RepositoryImpl};

#[async_trait]
impl AllGameCacheRepository for RepositoryImpl<AllGameCache> {
    async fn get_by_ids(
        &self,
        ids: Vec<i32>,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>> {
        let pool = self.pool.0.clone();
        let mut builder =
            sqlx::query_builder::QueryBuilder::new("SELECT * from all_game_caches WHERE id IN (");
        let mut separated = builder.separated(", ");
        for id in ids.iter() {
            separated.push_bind(id);
        }
        separated.push_unseparated(")");
        let query = builder.build();
        Ok(query
            .fetch_all(&*pool)
            .await?
            .into_iter()
            .map(|v| AllGameCacheOneWithThumbnailUrl {
                id: v.get(0),
                gamename: v.get(1),
                thumbnail_url: v.get(2),
            })
            .collect())
    }
    async fn get_all(&self) -> anyhow::Result<AllGameCache> {
        let pool = self.pool.0.clone();
        Ok(
            query_as::<_, AllGameCacheTable>("select * from all_game_caches")
                .fetch_all(&*pool)
                .await?
                .into_iter()
                .filter_map(|v| v.try_into().ok())
                .collect(),
        )
    }
    async fn get_last_updated(&self) -> anyhow::Result<(i32, DateTime<Local>)> {
        let pool = self.pool.0.clone();
        let last_updated: (i32, NaiveDateTime) =
            sqlx::query_as("SELECT MAX(id), MAX(created_at) from all_game_caches")
                .fetch_one(&*pool)
                .await?;
        Ok((
            last_updated.0,
            last_updated.1.and_utc().with_timezone(&Local),
        ))
    }
    async fn update(&self, cache: Vec<NewAllGameCacheOne>) -> anyhow::Result<()> {
        if cache.len() == 0 {
            return Ok(());
        }
        for c in cache.chunks(1000) {
            // ref: https://docs.rs/sqlx-core/latest/sqlx_core/query_builder/struct.QueryBuilder.html#method.push_values
            let mut query_builder =
                QueryBuilder::new("INSERT INTO all_game_caches (id, gamename, thumbnail_url) ");
            query_builder.push_values(c, |mut b, new| {
                b.push_bind(new.id);
                b.push_bind(new.gamename.clone());
                b.push_bind(new.thumbnail_url.clone());
            });

            let pool = self.pool.0.clone();
            let query = query_builder.build();
            query.execute(&*pool).await?;
        }
        Ok(())
    }
}
