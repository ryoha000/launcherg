use typeshare::typeshare;

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GetDmmOmitWorksRequestTs {
    pub extension_id: String,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DmmOmitDmmPartTs {
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DmmOmitWorkItemTs {
    pub work_id: i32,
    pub dmm: DmmOmitDmmPartTs,
}

