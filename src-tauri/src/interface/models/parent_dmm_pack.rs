use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmmPackKeysVm {
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
}


