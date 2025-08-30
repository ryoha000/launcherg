use crate::explored_cache::ExploredCache;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait ExploredCacheRepository {
    async fn get_all(&mut self) -> anyhow::Result<ExploredCache>;
    async fn add(&mut self, cache: ExploredCache) -> anyhow::Result<()>;
}


