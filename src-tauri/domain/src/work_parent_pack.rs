use serde::{Deserialize, Serialize};
use crate::works::Work;
use crate::Id;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkParentPack {
    pub id: Id<WorkParentPack>,
    pub work_id: Id<Work>,
    pub parent_pack_work_id: Id<Work>,
    pub created_at: String,
    pub updated_at: String,
}
