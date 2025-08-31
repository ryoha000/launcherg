use serde::{Deserialize, Serialize};
use crate::Id;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkDownloadPath {
    pub id: Id<WorkDownloadPath>,
    pub work_id: Id<crate::works::Work>,
    pub download_path: String,
}
