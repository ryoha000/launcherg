use crate::{works::Work, Id, StrId};
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct WorkLnk {
    pub id: Id<WorkLnk>,
    pub work_id: StrId<Work>,
    pub lnk_path: String,
}

#[derive(Clone, Debug)]
pub struct NewWorkLnk {
    pub work_id: StrId<Work>,
    pub lnk_path: String,
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkLnkRepository {
    async fn find_by_id(&mut self, id: Id<WorkLnk>) -> Result<Option<WorkLnk>>;
    async fn list_by_work_id(&mut self, work_id: StrId<Work>) -> Result<Vec<WorkLnk>>;
    async fn insert(&mut self, new_lnk: &NewWorkLnk) -> Result<Id<WorkLnk>>;
    async fn delete(&mut self, id: Id<WorkLnk>) -> Result<()>;
}
