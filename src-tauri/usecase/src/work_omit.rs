use std::sync::Arc;

use derive_new::new;
use domain::{
    work_omit::WorkOmit,
    works::Work,
    Id,
};
use domain::repositoryv2::{RepositoriesExt, work_omit::WorkOmitRepository};

#[derive(new)]
pub struct WorkOmitUseCase<R: RepositoriesExt> {
    repositories: Arc<tokio::sync::Mutex<R>>,
}

impl<R: RepositoriesExt> WorkOmitUseCase<R> {
    pub async fn add(&self, work_id: Id<Work>) -> anyhow::Result<()> { let mut repos = self.repositories.lock().await; repos.work_omit().add(work_id).await }
    pub async fn remove(&self, work_id: Id<Work>) -> anyhow::Result<()> { let mut repos = self.repositories.lock().await; repos.work_omit().remove(work_id).await }
    pub async fn list(&self) -> anyhow::Result<Vec<WorkOmit>> { let mut repos = self.repositories.lock().await; repos.work_omit().list().await }
    pub async fn exists(&self, work_id: Id<Work>) -> anyhow::Result<bool> { let mut repos = self.repositories.lock().await; repos.work_omit().exists(work_id).await }
}


