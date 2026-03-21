use crate::works::Work;
use crate::{Id, StrId};
use serde::{Deserialize, Serialize};

pub type ParentPackKey = crate::works::DmmPackKey;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkParentPack {
    pub id: Id<WorkParentPack>,
    pub work_id: StrId<Work>,
    pub parent_pack: ParentPackKey,
    pub created_at: String,
    pub updated_at: String,
}
