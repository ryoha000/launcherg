use crate::{
    work_parent_pack::ParentPackKey,
    works::Work,
    StrId,
};
use anyhow::Result;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkParentPacksRepository {
    async fn add(
        &mut self,
        work_id: StrId<Work>,
        parent_pack: ParentPackKey,
    ) -> Result<()>;
    async fn exists(
        &mut self,
        work_id: StrId<Work>,
        parent_pack: ParentPackKey,
    ) -> Result<bool>;
    async fn find_parent_key(&mut self, work_id: StrId<Work>) -> Result<Option<ParentPackKey>>;
}
