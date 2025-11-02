use typeshare::typeshare;

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DmmGameTs {
    pub id: String,
    pub category: String,
    pub subcategory: String,
    pub title: String,
    pub image_url: String,
    pub egs_info: Option<EgsInfoTs>,
    pub parent_pack_work_id: Option<String>,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DlsiteGameTs {
    pub id: String,
    pub category: String,
    pub title: String,
    pub image_url: String,
    pub egs_info: Option<EgsInfoTs>,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct EgsInfoTs {
    pub erogamescape_id: i32,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DmmSyncGamesRequestTs {
    pub games: Vec<DmmGameTs>,
    pub extension_id: String,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct DlsiteSyncGamesRequestTs {
    pub games: Vec<DlsiteGameTs>,
    pub extension_id: String,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SyncBatchResultTs {
    pub success_count: u32,
    pub error_count: u32,
    pub errors: Vec<String>,
    pub synced_games: Vec<String>,
}
