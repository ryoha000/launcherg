use std::sync::Arc;

use derive_new::new;

use domain::explored_cache::ExploredCache;
use domain::repository::{
    explored_cache::ExploredCacheRepository, manager::RepositoryManager, RepositoriesExt,
};
use std::marker::PhantomData;

#[derive(new)]
pub struct ExploredCacheUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    #[new(default)]
    _marker: PhantomData<R>,
}

impl<M, R> ExploredCacheUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn get_cache(&self) -> anyhow::Result<ExploredCache> {
        self.manager
            .run(|repos| Box::pin(async move { repos.explored_cache().get_all().await }))
            .await
    }
    pub async fn add_cache(&self, adding_path: Vec<String>) -> anyhow::Result<()> {
        let before = self
            .manager
            .run(|repos| Box::pin(async move { repos.explored_cache().get_all().await }))
            .await?;
        let adding = adding_path
            .into_iter()
            .filter_map(|v| match before.contains(&v) {
                true => None,
                false => Some(v),
            })
            .collect();
        self.manager
            .run(|repos| Box::pin(async move { repos.explored_cache().add(adding).await }))
            .await
    }
}
