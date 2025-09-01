use typeshare::typeshare;

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DownloadCompletedTs {
    pub id: i32,
    pub filename: String,
    pub mime: Option<String>,
    pub url: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DownloadIntentTs {
    pub store: String,
    pub game_store_id: String,
    pub game_category: String,
    pub game_subcategory: String,
    pub parent_pack_store_id: Option<String>,
    pub parent_pack_category: Option<String>,
    pub parent_pack_subcategory: Option<String>,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DownloadsCompletedRequestTs {
    pub extension_id: String,
    pub items: Vec<DownloadCompletedTs>,
    pub intent: Option<DownloadIntentTs>,
}


