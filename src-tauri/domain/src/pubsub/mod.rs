use derive_new::new;
use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(new, Clone, Serialize)]
pub struct ProgressPayload {
    pub message: String,
}

#[derive(new, Clone, Serialize)]
pub struct ProgressLivePayload {
    pub max: Option<i32>,
}

// スキャンFSM用イベントPayload
#[derive(new, Clone, Serialize)]
pub struct ScanProgressPayload {
    pub phase: String,
    pub processed: i32,
    pub total: i32,
    pub errors: i32,
    pub message: Option<String>,
}

#[derive(new, Clone, Serialize)]
pub struct ScanLogPayload {
    pub level: String,
    pub message: String,
}

#[derive(new, Clone, Serialize)]
pub struct ScanSummaryPayload {
    pub duration_ms: i64,
    pub found: i32,
    pub recognized: i32,
    pub persisted: i32,
    pub skipped: i32,
    pub duplicates: i32,
}

#[derive(Clone, Serialize)]
pub struct ExtensionConnectionPayload {
    pub connection_status: String, // ExtensionConnectionStatus のシリアライズ形式
    pub is_running: bool,
    pub error_message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

pub trait PubSubService: Send + Sync {
    fn notify<T: Serialize + Clone>(&self, event: &str, payload: T) -> Result<(), anyhow::Error>;
}
