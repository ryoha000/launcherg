use crate::all_game_cache::{AllGameCache, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne};
use chrono::{DateTime, Local};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait AllGameCacheRepository {
    async fn get_by_ids(
        &mut self,
        ids: Vec<i32>,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>>;
    async fn get_all(&mut self) -> anyhow::Result<AllGameCache>;
    async fn get_last_updated(&mut self) -> anyhow::Result<(i32, DateTime<Local>)>;
    async fn update(&mut self, cache: Vec<NewAllGameCacheOne>) -> anyhow::Result<()>;
    async fn delete_by_ids(&mut self, ids: Vec<i32>) -> anyhow::Result<()>;
    async fn search_by_name(
        &mut self,
        name: &str,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>>;
}
