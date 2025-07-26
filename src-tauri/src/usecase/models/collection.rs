use derive_new::new;
use serde::{Deserialize, Serialize};


#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionElementDetail {
    pub collection_element_id: i32,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

