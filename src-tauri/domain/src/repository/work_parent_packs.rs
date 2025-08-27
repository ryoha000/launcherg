use anyhow::Result;
use crate::{Id, works::Work};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkParentPacksRepository {
    async fn add(&self, work_id: Id<Work>, parent_pack_work_id: Id<Work>) -> Result<()>;
    async fn exists(&self, work_id: Id<Work>, parent_pack_work_id: Id<Work>) -> Result<bool>;
}


