use async_trait::async_trait;

use crate::domain::explored_cache::ExploredCache;

#[async_trait]
pub trait ExploredCacheRepository {
    async fn get_all(&self) -> anyhow::Result<ExploredCache>;
    async fn add(&self, cache: ExploredCache) -> anyhow::Result<()>;
}
