use chrono::{DateTime, Local};

use crate::all_game_cache::{
    AllGameCache, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne,
};

#[trait_variant::make(Send)]
#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
pub trait AllGameCacheRepository {
    async fn get_by_ids(
        &self,
        ids: Vec<i32>,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>>;
    async fn get_all(&self) -> anyhow::Result<AllGameCache>;
    async fn get_last_updated(&self) -> anyhow::Result<(i32, DateTime<Local>)>;
    async fn update(&self, cache: Vec<NewAllGameCacheOne>) -> anyhow::Result<()>;
    async fn delete_by_ids(&self, ids: Vec<i32>) -> anyhow::Result<()>;
    async fn search_by_name(&self, name: &str) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>>;
}
