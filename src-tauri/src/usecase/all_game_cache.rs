use std::sync::Arc;

use chrono::{DateTime, Local};
use derive_new::new;

use crate::{
    domain::{all_game_cache::AllGameCache, repository::all_game_cache::AllGameCacheRepository},
    infrastructure::repositoryimpl::repository::RepositoriesExt,
};

#[derive(new)]
pub struct AllGameCacheUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> AllGameCacheUseCase<R> {
    pub async fn get_all_game_cache(&self) -> anyhow::Result<AllGameCache> {
        self.repositories
            .all_game_cache_repository()
            .get_all()
            .await
    }
    pub async fn get_cache_last_updated(&self) -> anyhow::Result<(i32, DateTime<Local>)> {
        self.repositories
            .all_game_cache_repository()
            .get_last_updated()
            .await
    }
    pub async fn update_all_game_cache(&self, cache: AllGameCache) -> anyhow::Result<()> {
        self.repositories
            .all_game_cache_repository()
            .update(cache)
            .await
    }
}
