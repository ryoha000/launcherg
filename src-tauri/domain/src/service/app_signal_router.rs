use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// ネイティブホストなど外部コンポーネントからデスクトップアプリへ送るシグナル。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppSignalSource {
    NativeMessagingHost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AppSignalEvent {
    ShowErrorMessage {
        message: String,
    },
    ShowMessage {
        message: String,
    },
    RefetchWork {
        #[serde(rename = "workId")]
        work_id: String,
    },
    RefetchWorks,
    SyncRequested {
        #[serde(default)]
        message: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSignal {
    pub source: AppSignalSource,
    pub event: AppSignalEvent,
    pub issued_at: DateTime<Utc>,
}

#[trait_variant::make(Send + Sync)]
#[mockall::automock]
pub trait AppSignalRouter {
    /// シグナルをデスクトップアプリへ中継する。
    async fn dispatch(&self, signal: AppSignal) -> anyhow::Result<()>;
}
