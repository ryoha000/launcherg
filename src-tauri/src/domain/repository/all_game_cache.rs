use async_trait::async_trait;
use chrono::{DateTime, Local};

use crate::domain::all_game_cache::AllGameCache;

#[async_trait]
pub trait AllGameCacheRepository {
    async fn get_all(&self) -> anyhow::Result<AllGameCache>;
    async fn get_last_updated(&self) -> anyhow::Result<(i32, DateTime<Local>)>;
    async fn update(&self, cache: AllGameCache) -> anyhow::Result<()>;
}
