use std::sync::Arc;

use derive_new::new;

use domain::explored_cache::ExploredCache;
use domain::repository::{explored_cache::ExploredCacheRepository, RepositoriesExt};

#[derive(new)]
pub struct ExploredCacheUseCase<R: RepositoriesExt> {
    repositories: Arc<tokio::sync::Mutex<R>>,
}

impl<R: RepositoriesExt> ExploredCacheUseCase<R> {
    pub async fn get_cache(&self) -> anyhow::Result<ExploredCache> {
        let mut repos = self.repositories.lock().await;
        Ok(repos
            .explored_cache()
            .get_all()
            .await?)
    }
    pub async fn add_cache(&self, adding_path: Vec<String>) -> anyhow::Result<()> {
        let before = {
            let mut repos = self.repositories.lock().await;
            repos
                .explored_cache()
                .get_all()
                .await?
        };
        let adding = adding_path
            .into_iter()
            .filter_map(|v| match before.contains(&v) {
                true => None,
                false => Some(v),
            })
            .collect();
        let mut repos = self.repositories.lock().await;
        Ok(repos
            .explored_cache()
            .add(adding)
            .await?)
    }
}
