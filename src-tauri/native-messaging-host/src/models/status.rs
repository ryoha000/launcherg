use typeshare::typeshare;

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ExtensionConfigTs {
    pub auto_sync: bool,
    pub allowed_domains: Vec<String>,
    pub sync_interval_minutes: u32,
    pub debug_mode: bool,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SyncStatusTs {
    pub last_sync: Option<TimestampTs>,
    pub total_synced: u32,
    pub connected_extensions: u32,
    pub is_running: bool,
    pub connection_status: String,
    pub error_message: String,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct TimestampTs {
    pub seconds: i32,
    pub nanos: i32,
}

#[typeshare]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ConfigUpdateResultTs {
    pub message: String,
}
