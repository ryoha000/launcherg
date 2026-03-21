use std::sync::Arc;

use derive_new::new;

use domain::erogamescape::NewErogamescapeInformation;
use domain::repository::{
    erogamescape::ErogamescapeRepository, manager::RepositoryManager, RepositoriesExt,
};

#[derive(new)]
pub struct ErogamescapeUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    #[new(default)]
    _marker: std::marker::PhantomData<R>,
}

impl<M, R> ErogamescapeUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn upsert_information(
        &self,
        info: &NewErogamescapeInformation,
    ) -> anyhow::Result<()> {
        self.manager
            .run(|repos| {
                let info = info.clone();
                Box::pin(async move { repos.erogamescape().upsert_information(&info).await })
            })
            .await?;
        Ok(())
    }

    pub async fn upsert_information_batch(
        &self,
        infos: &[NewErogamescapeInformation],
    ) -> anyhow::Result<()> {
        for info in infos.iter() {
            self.upsert_information(info).await?;
        }
        Ok(())
    }

    pub async fn find_missing_information_ids(&self) -> anyhow::Result<Vec<i32>> {
        self.manager
            .run(|repos| {
                Box::pin(async move { repos.erogamescape().find_missing_information_ids().await })
            })
            .await
    }
}
