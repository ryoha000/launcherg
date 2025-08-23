use anyhow::Result;
use crate::deny_list::{DenyListEntry, StoreType};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DenyListRepository {
    async fn add(&self, store_type: StoreType, store_id: &str, name: &str) -> Result<()>;
    async fn remove(&self, store_type: StoreType, store_id: &str) -> Result<()>;
    async fn list(&self) -> Result<Vec<DenyListEntry>>;
    async fn exists(&self, store_type: StoreType, store_id: &str) -> Result<bool>;
}


