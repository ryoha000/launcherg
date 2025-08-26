use serde::{Deserialize, Serialize};
use crate::Id;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkOmit { pub id: Id<WorkOmit>, pub work_id: Id<crate::works::Work> }


