use std::sync::Arc;

use derive_new::new;
use domain::{
    repository::{works::WorkRepository, RepositoriesExt},
    works::WorkDetails,
};

#[derive(new)]
pub struct WorkUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> WorkUseCase<R> {
    pub async fn list_all_details(&self) -> anyhow::Result<Vec<WorkDetails>> {
        self.repositories.work_repository().list_all_details().await
    }
}


