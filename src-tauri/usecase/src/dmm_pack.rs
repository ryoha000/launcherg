use std::sync::Arc;

use derive_new::new;
use domain::{
    dmm_work_pack::DmmWorkPack,
    works::Work,
    Id,
};
use domain::repository::{dmm_work_pack::DmmPackRepository, RepositoriesExt};

#[derive(new)]
pub struct DmmPackUseCase<R: RepositoriesExt> {
    repositories: Arc<tokio::sync::Mutex<R>>,
}

impl<R: RepositoriesExt> DmmPackUseCase<R> {
    pub async fn add(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        let mut repos = self.repositories.lock().await;
        repos.dmm_pack().add(work_id).await
    }

    pub async fn remove(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        let mut repos = self.repositories.lock().await;
        repos.dmm_pack().remove(work_id).await
    }

    pub async fn list(&self) -> anyhow::Result<Vec<DmmWorkPack>> {
        let mut repos = self.repositories.lock().await;
        repos.dmm_pack().list().await
    }
}


