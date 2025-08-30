use anyhow::Result;
use crate::{dmm_work_pack::DmmWorkPack, works::Work, Id};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DmmPackRepository {
    async fn add(&mut self, work_id: Id<Work>) -> Result<()>;
    async fn remove(&mut self, work_id: Id<Work>) -> Result<()>;
    async fn list(&mut self) -> Result<Vec<DmmWorkPack>>;
    async fn exists(&mut self, work_id: Id<Work>) -> Result<bool>;
}


