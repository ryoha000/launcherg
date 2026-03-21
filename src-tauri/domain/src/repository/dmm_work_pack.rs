use crate::{dmm_work_pack::DmmWorkPack, works::Work, StrId};
use anyhow::Result;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DmmPackRepository {
    async fn add(&mut self, work_id: StrId<Work>) -> Result<()>;
    async fn remove(&mut self, work_id: StrId<Work>) -> Result<()>;
    async fn list(&mut self) -> Result<Vec<DmmWorkPack>>;
    async fn exists(&mut self, work_id: StrId<Work>) -> Result<bool>;
}
