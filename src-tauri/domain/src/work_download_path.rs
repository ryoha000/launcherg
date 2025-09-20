use crate::Id;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkDownloadPath {
    pub id: Id<WorkDownloadPath>,
    pub work_id: Id<crate::works::Work>,
    pub download_path: String,
}
