use std::sync::Arc;

use derive_new::new;
use domain::{
    dmm_work_pack::DmmWorkPack,
    repository::{dmm_work_pack::DmmPackRepository, RepositoriesExt},
    works::Work,
    Id,
};

#[derive(new)]
pub struct DmmPackUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> DmmPackUseCase<R> {
    pub async fn add(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        self.repositories.dmm_pack().add(work_id).await
    }

    pub async fn remove(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        self.repositories.dmm_pack().remove(work_id).await
    }

    pub async fn list(&self) -> anyhow::Result<Vec<DmmWorkPack>> {
        self.repositories.dmm_pack().list().await
    }
}


