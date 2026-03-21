use crate::{works::Work, Id, StrId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmmWorkPack {
    pub id: Id<DmmWorkPack>,
    pub work_id: StrId<Work>,
}
