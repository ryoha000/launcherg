use anyhow::Result;

use crate::dmm_pack::DmmPackMark;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DmmPackRepository {
    async fn add(&self, store_id: &str, name: &str) -> Result<()>;
    async fn remove(&self, store_id: &str) -> Result<()>;
    async fn list(&self) -> Result<Vec<DmmPackMark>>;
    async fn exists(&self, store_id: &str) -> Result<bool>;
}


