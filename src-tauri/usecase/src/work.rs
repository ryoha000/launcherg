use std::sync::Arc;

use derive_new::new;
use domain::{
    works::WorkDetails,
};
use domain::repositoryv2::{works::WorkRepository, RepositoriesExt};

#[derive(new)]
pub struct WorkUseCase<R: RepositoriesExt> {
    repositories: Arc<tokio::sync::Mutex<R>>,
}

impl<R: RepositoriesExt> WorkUseCase<R> {
    pub async fn list_all_details(&self) -> anyhow::Result<Vec<WorkDetails>> {
        let mut repos = self.repositories.lock().await;
        repos.work().list_all_details().await
    }
}


