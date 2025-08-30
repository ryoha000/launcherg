use std::sync::Arc;

use chrono::{DateTime, Local};
use derive_new::new;

use domain::{
    all_game_cache::{AllGameCache, AllGameCacheOneWithThumbnailUrl, NewAllGameCacheOne},
};
use domain::repositoryv2::{RepositoriesExt, all_game_cache::AllGameCacheRepository};

#[derive(new)]
pub struct AllGameCacheUseCase<R: RepositoriesExt> {
    repositories: Arc<tokio::sync::Mutex<R>>,
}

impl<R: RepositoriesExt> AllGameCacheUseCase<R> {
    pub async fn get(&self, id: i32) -> anyhow::Result<Option<AllGameCacheOneWithThumbnailUrl>> {
        let mut repos = self.repositories.lock().await;
        Ok(repos
            .all_game_cache()
            .get_by_ids(vec![id])
            .await?
            .first()
            .and_then(|v| Some(v.clone())))
    }
    pub async fn get_by_ids(
        &self,
        ids: Vec<i32>,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>> {
        let mut repos = self.repositories.lock().await;
        repos
            .all_game_cache()
            .get_by_ids(ids)
            .await
    }
    pub async fn get_all_game_cache(&self) -> anyhow::Result<AllGameCache> {
        let mut repos = self.repositories.lock().await;
        repos
            .all_game_cache()
            .get_all()
            .await
    }
    pub async fn get_cache_last_updated(&self) -> anyhow::Result<(i32, DateTime<Local>)> {
        let mut repos = self.repositories.lock().await;
        repos
            .all_game_cache()
            .get_last_updated()
            .await
    }
    pub async fn update_all_game_cache(
        &self,
        cache: Vec<NewAllGameCacheOne>,
    ) -> anyhow::Result<()> {
        if cache.is_empty() {
            return Ok(());
        }
        {
            let mut repos = self.repositories.lock().await;
            repos
                .all_game_cache()
                .delete_by_ids(cache.iter().map(|v| v.id).collect())
                .await?;
        }
        let mut repos = self.repositories.lock().await;
        repos
            .all_game_cache()
            .update(cache)
            .await
    }

    pub async fn search_games_by_name(
        &self,
        name: &str,
    ) -> anyhow::Result<Vec<AllGameCacheOneWithThumbnailUrl>> {
        let mut repos = self.repositories.lock().await;
        repos
            .all_game_cache()
            .search_by_name(name)
            .await
    }
}
