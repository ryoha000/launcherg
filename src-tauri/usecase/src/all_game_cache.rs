use std::sync::Arc;

use chrono::{DateTime, Local};
use derive_new::new;

use domain::{
    all_game_cache::{AllGameCache, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne},
};
use domain::repository::{RepositoriesExt, all_game_cache::AllGameCacheRepository, manager::RepositoryManager};
use std::marker::PhantomData;

#[derive(new)]
pub struct AllGameCacheUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    #[new(default)] _marker: PhantomData<R>,
}

impl<M, R> AllGameCacheUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn get(&self, id: i32) -> anyhow::Result<Option<AllGameCacheOneWithThumbnailUrl>> {
        self.manager.run(move |repos| {
            Box::pin(async move {
                Ok(repos
                    .all_game_cache()
                    .get_by_ids(vec![id])
                    .await?
                    .first()
                    .cloned())
            })
        }).await
    }
    pub async fn get_by_ids(
        &self,
        ids: Vec<i32>,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>> {
        self.manager.run(move |repos| {
            Box::pin(async move { repos.all_game_cache().get_by_ids(ids).await })
        }).await
    }
    pub async fn get_all_game_cache(&self) -> anyhow::Result<AllGameCache> {
        self.manager.run(|repos| Box::pin(async move { repos.all_game_cache().get_all().await })).await
    }
    pub async fn get_cache_last_updated(&self) -> anyhow::Result<(i32, DateTime<Local>)> {
        self.manager.run(|repos| Box::pin(async move { repos.all_game_cache().get_last_updated().await })).await
    }
    pub async fn update_all_game_cache(
        &self,
        cache: Vec<NewAllGameCacheOne>,
    ) -> anyhow::Result<()> {
        if cache.is_empty() {
            return Ok(());
        }
        let ids: Vec<i32> = cache.iter().map(|v| v.id).collect();
        self.manager.run_in_transaction(move |repos| {
            let cache_clone = cache.clone();
            Box::pin(async move {
                repos.all_game_cache().delete_by_ids(ids).await?;
                repos.all_game_cache().update(cache_clone).await
            })
        }).await
    }

    pub async fn search_games_by_name(
        &self,
        name: &str,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>> {
        let name = name.to_string();
        self.manager.run(move |repos| Box::pin(async move { repos.all_game_cache().search_by_name(&name).await })).await
    }
}
