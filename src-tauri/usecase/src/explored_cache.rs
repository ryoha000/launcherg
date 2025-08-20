use std::sync::Arc;

use derive_new::new;

use domain::{explored_cache::ExploredCache, repository::explored_cache::ExploredCacheRepository};
use infrastructure::repositoryimpl::repository::RepositoriesExt;

#[derive(new)]
pub struct ExploredCacheUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> ExploredCacheUseCase<R> {
    pub async fn get_cache(&self) -> anyhow::Result<ExploredCache> {
        Ok(self
            .repositories
            .explored_cache_repository()
            .get_all()
            .await?)
    }
    pub async fn add_cache(&self, adding_path: Vec<String>) -> anyhow::Result<()> {
        let before = self
            .repositories
            .explored_cache_repository()
            .get_all()
            .await?;
        let adding = adding_path
            .into_iter()
            .filter_map(|v| match before.contains(&v) {
                true => None,
                false => Some(v),
            })
            .collect();
        Ok(self
            .repositories
            .explored_cache_repository()
            .add(adding)
            .await?)
    }
}
