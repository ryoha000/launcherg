use std::sync::Arc;

use derive_new::new;
use domain::{deny_list::{DenyListEntry, StoreType}, repository::{RepositoriesExt, deny_list::DenyListRepository}};

#[derive(new)]
pub struct DenyListUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> DenyListUseCase<R> {
    pub async fn add(&self, store_type: StoreType, store_id: &str, name: &str) -> anyhow::Result<()> {
        self.repositories.deny_list_repository().add(store_type, store_id, name).await
    }
    pub async fn remove(&self, store_type: StoreType, store_id: &str) -> anyhow::Result<()> {
        self.repositories.deny_list_repository().remove(store_type, store_id).await
    }
    pub async fn list(&self) -> anyhow::Result<Vec<DenyListEntry>> {
        self.repositories.deny_list_repository().list().await
    }
    pub async fn is_denied(&self, store_type: StoreType, store_id: &str) -> anyhow::Result<bool> {
        self.repositories.deny_list_repository().exists(store_type, store_id).await
    }
}


