use crate::{works::Work, Id};
use anyhow::Result;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkParentPacksRepository {
    async fn add(&mut self, work_id: Id<Work>, parent_pack_work_id: Id<Work>) -> Result<()>;
    async fn exists(&mut self, work_id: Id<Work>, parent_pack_work_id: Id<Work>) -> Result<bool>;
    async fn find_parent_id(&mut self, work_id: Id<Work>) -> Result<Option<Id<Work>>>;
}
