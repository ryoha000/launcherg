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
#[serde(tag = "case", content = "value")]
pub enum DownloadIntentTs {
    Dmm {
        game_store_id: String,
        game_category: String,
        game_subcategory: String,
        parent_pack_store_id: Option<String>,
        parent_pack_category: Option<String>,
        parent_pack_subcategory: Option<String>,
    },
    Dlsite {
        game_store_id: String,
        game_category: String,
    },
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DownloadsCompletedRequestTs {
    pub extension_id: String,
    pub items: Vec<DownloadCompletedTs>,
    pub intent: DownloadIntentTs,
}
