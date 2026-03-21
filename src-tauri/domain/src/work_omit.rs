use crate::{Id, StrId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkOmit {
    pub id: Id<WorkOmit>,
    pub work_id: StrId<crate::works::Work>,
}
