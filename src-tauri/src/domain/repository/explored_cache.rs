use crate::domain::explored_cache::ExploredCache;

#[cfg_attr(test, mockall::automock)]
pub trait ExploredCacheRepository {
    async fn get_all(&self) -> anyhow::Result<ExploredCache>;
    async fn add(&self, cache: ExploredCache) -> anyhow::Result<()>;
}
