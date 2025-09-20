use crate::Id;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkOmit {
    pub id: Id<WorkOmit>,
    pub work_id: Id<crate::works::Work>,
}
