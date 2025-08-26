use std::sync::Arc;

use derive_new::new;
use domain::{
    work_omit::WorkOmit,
    repository::RepositoriesExt,
    works::Work,
    Id,
};
use domain::repository::work_omit::WorkOmitRepository;

#[derive(new)]
pub struct WorkOmitUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> WorkOmitUseCase<R> {
    pub async fn add(&self, work_id: Id<Work>) -> anyhow::Result<()> { self.repositories.work_omit_repository().add(work_id).await }
    pub async fn remove(&self, work_id: Id<Work>) -> anyhow::Result<()> { self.repositories.work_omit_repository().remove(work_id).await }
    pub async fn list(&self) -> anyhow::Result<Vec<WorkOmit>> { self.repositories.work_omit_repository().list().await }
    pub async fn exists(&self, work_id: Id<Work>) -> anyhow::Result<bool> { self.repositories.work_omit_repository().exists(work_id).await }
}


