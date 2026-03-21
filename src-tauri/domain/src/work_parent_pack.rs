use crate::works::Work;
use crate::{Id, StrId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkParentPack {
    pub id: Id<WorkParentPack>,
    pub work_id: StrId<Work>,
    pub parent_pack_work_id: StrId<Work>,
    pub created_at: String,
    pub updated_at: String,
}
