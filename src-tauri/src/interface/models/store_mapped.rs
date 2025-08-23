use serde::Serialize;

// no imports needed here

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoreMappedElementVm {
    pub collection_element_id: i32,
    pub store_type: i32,
    pub store_id: String,
    pub title: String,
    pub brand: String,
    pub dmm_category: Option<String>,
    pub dmm_subcategory: Option<String>,
    pub dlsite_category: Option<String>,
    pub already_denied: bool,
    pub is_dmm_pack: bool,
    pub thumbnail: String,
}



