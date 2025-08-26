use anyhow::Result;

use crate::{dmm_work_pack::DmmWorkPack, works::Work, Id};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DmmPackRepository {
    async fn add(&self, work_id: Id<Work>) -> Result<()>;
    async fn remove(&self, work_id: Id<Work>) -> Result<()>;
    async fn list(&self) -> Result<Vec<DmmWorkPack>>;
    async fn exists(&self, work_id: Id<Work>) -> Result<bool>;
}


