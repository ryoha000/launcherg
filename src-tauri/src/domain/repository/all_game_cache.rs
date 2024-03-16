use async_trait::async_trait;
use chrono::{DateTime, Local};

use crate::domain::all_game_cache::{
    AllGameCache, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne,
};

#[async_trait]
pub trait AllGameCacheRepository {
    async fn get_by_ids(
        &self,
        ids: Vec<i32>,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>>;
    async fn get_all(&self) -> anyhow::Result<AllGameCache>;
    async fn get_last_updated(&self) -> anyhow::Result<(i32, DateTime<Local>)>;
    async fn update(&self, cache: Vec<NewAllGameCacheOne>) -> anyhow::Result<()>;
    async fn delete_by_ids(&self, ids: Vec<i32>) -> anyhow::Result<()>;
}
