use std::sync::Arc;

use derive_new::new;
use domain::repository::{
    manager::RepositoryManager, work_omit::WorkOmitRepository, RepositoriesExt,
};
use domain::{work_omit::WorkOmit, works::Work, Id};
use std::marker::PhantomData;

#[derive(new)]
pub struct WorkOmitUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    _marker: PhantomData<R>,
}

impl<M, R> WorkOmitUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn add(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        self.manager
            .run(move |repos| Box::pin(async move { repos.work_omit().add(work_id).await }))
            .await
    }
    pub async fn remove(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        self.manager
            .run(move |repos| Box::pin(async move { repos.work_omit().remove(work_id).await }))
            .await
    }
    pub async fn list(&self) -> anyhow::Result<Vec<WorkOmit>> {
        self.manager
            .run(|repos| Box::pin(async move { repos.work_omit().list().await }))
            .await
    }
    pub async fn exists(&self, work_id: Id<Work>) -> anyhow::Result<bool> {
        self.manager
            .run(move |repos| Box::pin(async move { repos.work_omit().exists(work_id).await }))
            .await
    }
}
