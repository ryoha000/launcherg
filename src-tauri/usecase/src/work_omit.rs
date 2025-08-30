use std::sync::Arc;

use derive_new::new;
use domain::{
    work_omit::WorkOmit,
    repository::RepositoriesExt,
    works::Work,
    Id,
};
use domain::repositoryv2::work_omit::WorkOmitRepository;

#[derive(new)]
pub struct WorkOmitUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> WorkOmitUseCase<R> {
    pub async fn add(&self, work_id: Id<Work>) -> anyhow::Result<()> { self.repositories.work_omit().add(work_id).await }
    pub async fn remove(&self, work_id: Id<Work>) -> anyhow::Result<()> { self.repositories.work_omit().remove(work_id).await }
    pub async fn list(&self) -> anyhow::Result<Vec<WorkOmit>> { self.repositories.work_omit().list().await }
    pub async fn exists(&self, work_id: Id<Work>) -> anyhow::Result<bool> { self.repositories.work_omit().exists(work_id).await }
}


