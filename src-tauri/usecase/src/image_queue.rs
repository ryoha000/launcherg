use std::sync::Arc;

use derive_new::new;
use domain::repository::{
    manager::RepositoryManager, save_image_queue::ImageSaveQueueRepository as _, RepositoriesExt,
};
use std::marker::PhantomData;

#[derive(new)]
pub struct ImageQueueUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    _marker: PhantomData<R>,
}

impl<M, R> ImageQueueUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn list(
        &self,
        unfinished: bool,
        limit: i64,
    ) -> anyhow::Result<Vec<domain::save_image_queue::ImageSaveQueueRow>> {
        self.manager
            .run(|repos| Box::pin(async move { repos.image_queue().list(unfinished, limit).await }))
            .await
    }
}
