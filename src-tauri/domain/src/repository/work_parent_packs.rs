use crate::{works::Work, StrId};
use anyhow::Result;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkParentPacksRepository {
    async fn add(&mut self, work_id: StrId<Work>, parent_pack_work_id: StrId<Work>) -> Result<()>;
    async fn exists(
        &mut self,
        work_id: StrId<Work>,
        parent_pack_work_id: StrId<Work>,
    ) -> Result<bool>;
    async fn find_parent_id(&mut self, work_id: StrId<Work>) -> Result<Option<StrId<Work>>>;
}
