use crate::explored_cache::ExploredCache;

#[trait_variant::make(Send)]
#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
pub trait ExploredCacheRepository {
    async fn get_all(&self) -> anyhow::Result<ExploredCache>;
    async fn add(&self, cache: ExploredCache) -> anyhow::Result<()>;
}
