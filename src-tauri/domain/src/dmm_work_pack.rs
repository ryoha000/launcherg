use serde::{Deserialize, Serialize};
use crate::{Id, works::Work};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmmWorkPack {
    pub id: Id<DmmWorkPack>,
    pub work_id: Id<Work>,
}


