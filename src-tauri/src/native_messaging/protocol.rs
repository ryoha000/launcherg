use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeMessage {
    #[serde(rename = "type")]
    pub type_: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    SyncGames,
    GetStatus,
    SetConfig,
    HealthCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncGamesRequest {
    pub store: String, // "DMM" | "DLSite"
    pub games: Vec<ExtractedGameData>,
    pub extension_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedGameData {
    pub store_id: String,
    pub title: String,
    pub purchase_url: String,
    pub purchase_date: Option<String>,
    pub thumbnail_url: Option<String>,
    pub additional_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncBatchResult {
    pub success_count: u32,
    pub error_count: u32,
    pub errors: Vec<String>,
    pub synced_games: Vec<String>, // ゲームタイトル
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_sync: Option<DateTime<Utc>>,
    pub total_synced: u32,
    pub connected_extensions: Vec<String>,
    pub is_running: bool,
    pub connection_status: ExtensionConnectionStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionConnectionStatus {
    /// 正常に接続済み
    Connected,
    /// 接続中（チェック実行中）
    Connecting,
    /// Native Messaging Hostの実行ファイルが見つからない
    HostNotFound,
    /// Native Messaging Hostプロセスの起動に失敗
    HostStartupFailed,
    /// ヘルスチェック通信のタイムアウト
    HealthCheckTimeout,
    /// ヘルスチェック通信でエラーレスポンス
    HealthCheckFailed,
    /// JSON解析エラーなどの通信エラー
    CommunicationError,
    /// プロセス終了時のエラー
    ProcessTerminationError,
    /// 不明なエラー
    UnknownError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionConfig {
    pub auto_sync: bool,
    pub allowed_domains: Vec<String>,
    pub sync_interval_minutes: u32,
    pub debug_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub request_id: String,
}

impl NativeMessage {
    pub fn new(type_: MessageType, payload: serde_json::Value) -> Self {
        Self {
            type_,
            payload,
            timestamp: Utc::now(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl NativeResponse {
    pub fn success(data: serde_json::Value, request_id: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            request_id,
        }
    }

    pub fn error(error: String, request_id: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            request_id,
        }
    }
}