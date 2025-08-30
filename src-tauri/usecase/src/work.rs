use std::sync::Arc;

use derive_new::new;
use domain::{
    works::WorkDetails,
};
use domain::repository::{works::WorkRepository, RepositoriesExt, manager::RepositoryManager};
use std::marker::PhantomData;

#[derive(new)]
pub struct WorkUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    #[new(default)] _marker: PhantomData<R>,
}

impl<M, R> WorkUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn list_all_details(&self) -> anyhow::Result<Vec<WorkDetails>> {
        self.manager.run(|repos| {
            Box::pin(async move { repos.work().list_all_details().await })
        }).await
    }
}


