use typeshare::typeshare;

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GetDmmPackIdsRequestTs {
    pub extension_id: String,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DmmPackIdsResponseTs {
    pub store_ids: Vec<String>,
}

