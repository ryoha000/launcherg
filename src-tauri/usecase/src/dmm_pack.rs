use std::sync::Arc;

use derive_new::new;
use domain::{
    dmm_work_pack::DmmWorkPack,
    works::Work,
    Id,
};
use domain::repository::{dmm_work_pack::DmmPackRepository, RepositoriesExt, manager::RepositoryManager};
use std::marker::PhantomData;

#[derive(new)]
pub struct DmmPackUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    #[new(default)] _marker: PhantomData<R>,
}

impl<M, R> DmmPackUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn add(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        self.manager.run(move |repos| {
            Box::pin(async move {
                let mut repo = repos.dmm_pack();
                repo.add(work_id).await
            })
        }).await
    }

    pub async fn remove(&self, work_id: Id<Work>) -> anyhow::Result<()> {
        self.manager.run(move |repos| {
            Box::pin(async move {
                let mut repo = repos.dmm_pack();
                repo.remove(work_id).await
            })
        }).await
    }

    pub async fn list(&self) -> anyhow::Result<Vec<DmmWorkPack>> {
        self.manager.run(|repos| {
            Box::pin(async move {
                let mut repo = repos.dmm_pack();
                repo.list().await
            })
        }).await
    }
}


