use crate::{Id, StrId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkDownloadPath {
    pub id: Id<WorkDownloadPath>,
    pub work_id: StrId<crate::works::Work>,
    pub download_path: String,
}
