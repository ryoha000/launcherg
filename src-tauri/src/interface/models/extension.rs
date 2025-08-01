use serde::{Deserialize, Serialize};

/// 同期ステータス情報
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncStatus {
    /// 最後の同期時刻（optional）
    pub last_sync: Option<pbjson_types::Timestamp>,
    /// 同期したゲーム総数
    pub total_synced: u32, 
    /// 接続済み拡張機能のリスト
    pub connected_extensions: Vec<String>,
    /// Native Messaging Hostが動作中かどうか
    pub is_running: bool,
    /// 接続ステータス
    pub connection_status: i32,
    /// エラーメッセージ（optional）
    pub error_message: String,
}

/// 拡張機能の設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionConfig {
    /// 自動同期の有効・無効
    pub auto_sync: bool,
    /// 許可されたドメインのリスト
    pub allowed_domains: Vec<String>,
    /// 同期間隔（分）
    pub sync_interval_minutes: u32,
    /// デバッグモードの有効・無効
    pub debug_mode: bool,
}

/// 拡張機能の接続ステータス
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ExtensionConnectionStatus {
    Unspecified = 0,
    /// 正常に接続済み
    Connected = 1,
    /// 接続中（チェック実行中）
    Connecting = 2,
    /// Native Messaging Hostの実行ファイルが見つからない
    HostNotFound = 3,
    /// Native Messaging Hostの起動に失敗
    HostStartupFailed = 4,
    /// ヘルスチェックがタイムアウト
    HealthCheckTimeout = 5,
    /// ヘルスチェックに失敗
    HealthCheckFailed = 6,
    /// 通信エラー
    CommunicationError = 7,
    /// プロセス終了エラー
    ProcessTerminationError = 8,
    /// 不明なエラー
    UnknownError = 9,
}