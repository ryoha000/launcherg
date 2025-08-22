use serde::{Deserialize, Serialize};

use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmmPackMark {
    pub id: Id<DmmPackMark>,
    pub store_id: String,
}


