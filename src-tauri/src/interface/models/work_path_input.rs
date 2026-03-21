use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WorkPathInput {
    Exe { exe_path: String },
    Lnk { lnk_path: String },
}
