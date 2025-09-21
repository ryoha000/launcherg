use chrono::{DateTime, Utc};
use derive_new::new;
use serde::Serialize;
use typeshare::typeshare;

use crate::service::app_signal_router::{AppSignal, AppSignalEvent, AppSignalSource};

#[typeshare]
#[derive(new, Clone, Serialize)]
pub struct ProgressPayload {
    pub message: String,
}

#[typeshare]
#[derive(new, Clone, Serialize)]
pub struct ProgressLivePayload {
    pub max: Option<i32>,
}

// スキャンFSM用イベントPayload
#[typeshare]
#[derive(new, Clone, Serialize)]
pub struct ScanProgressPayload {
    pub phase: String,
    pub processed: i32,
    pub total: i32,
    pub errors: i32,
    pub message: Option<String>,
}

#[typeshare]
#[derive(new, Clone, Serialize)]
pub struct ScanLogPayload {
    pub level: String,
    pub message: String,
}

#[typeshare]
#[derive(new, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummaryPayload {
    pub duration_ms: i32,
    pub found: i32,
    pub recognized: i32,
    pub persisted: i32,
    pub skipped: i32,
    pub duplicates: i32,
}

#[typeshare]
#[derive(new, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanPhaseTimingPayload {
    pub phase: String,
    pub duration_ms: i32,
}

#[typeshare]
#[derive(new, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrichResultPayload {
    pub status: String, // "candidate" | "resolved"
    pub path: String,
    pub title: Option<String>,
    pub egs_id: Option<i32>,
}

#[typeshare]
#[derive(new, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupResultPayload {
    pub removed_count: i32,
}

#[typeshare]
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionConnectionPayload {
    pub connection_status: String, // ExtensionConnectionStatus のシリアライズ形式
    pub is_running: bool,
    pub error_message: Option<String>,
    #[typeshare(serialized_as = "String")]
    pub timestamp: DateTime<Utc>,
}

// ImageQueue 用のイベントペイロード
#[typeshare]
#[derive(new, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageQueueWorkerStatusPayload {
    pub status: String, // "started" | "finished"
}

#[typeshare]
#[derive(new, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageQueueItemPayload {
    pub id: String,
    pub src: String,
    pub src_type: i32,
    pub dst_path: String,
}

#[typeshare]
#[derive(new, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageQueueItemErrorPayload {
    pub id: String,
    pub message: String,
}

#[typeshare]
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSignalPayload {
    pub source: AppSignalSourcePayload,
    pub event: AppSignalEventPayload,
    #[typeshare(serialized_as = "String")]
    pub issued_at: DateTime<Utc>,
}

#[typeshare]
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AppSignalSourcePayload {
    NativeMessagingHost,
}

#[typeshare]
#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "payload", rename_all = "camelCase")]
pub enum AppSignalEventPayload {
    ShowErrorMessage {
        message: String,
    },
    ShowMessage {
        message: String,
    },
    SyncRequested {
        #[serde(default)]
        message: Option<String>,
    },
}

impl From<AppSignalSource> for AppSignalSourcePayload {
    fn from(value: AppSignalSource) -> Self {
        match value {
            AppSignalSource::NativeMessagingHost => AppSignalSourcePayload::NativeMessagingHost,
        }
    }
}

impl From<AppSignalSourcePayload> for AppSignalSource {
    fn from(value: AppSignalSourcePayload) -> Self {
        match value {
            AppSignalSourcePayload::NativeMessagingHost => AppSignalSource::NativeMessagingHost,
        }
    }
}

impl From<AppSignalEvent> for AppSignalEventPayload {
    fn from(value: AppSignalEvent) -> Self {
        match value {
            AppSignalEvent::ShowErrorMessage { message } => {
                AppSignalEventPayload::ShowErrorMessage { message }
            }
            AppSignalEvent::ShowMessage { message } => {
                AppSignalEventPayload::ShowMessage { message }
            }
            AppSignalEvent::SyncRequested { message } => {
                AppSignalEventPayload::SyncRequested { message }
            }
        }
    }
}

impl From<AppSignalEventPayload> for AppSignalEvent {
    fn from(value: AppSignalEventPayload) -> Self {
        match value {
            AppSignalEventPayload::ShowErrorMessage { message } => {
                AppSignalEvent::ShowErrorMessage { message }
            }
            AppSignalEventPayload::ShowMessage { message } => {
                AppSignalEvent::ShowMessage { message }
            }
            AppSignalEventPayload::SyncRequested { message } => {
                AppSignalEvent::SyncRequested { message }
            }
        }
    }
}

impl From<AppSignal> for AppSignalPayload {
    fn from(value: AppSignal) -> Self {
        Self {
            source: value.source.into(),
            event: value.event.into(),
            issued_at: value.issued_at,
        }
    }
}

impl From<AppSignalPayload> for AppSignal {
    fn from(value: AppSignalPayload) -> Self {
        Self {
            source: value.source.into(),
            event: value.event.into(),
            issued_at: value.issued_at,
        }
    }
}

#[typeshare]
#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "payload")]
pub enum PubSubEvent {
    #[serde(rename = "progress")]
    Progress(ProgressPayload),
    #[serde(rename = "progresslive")]
    ProgressLive(ProgressLivePayload),
    #[serde(rename = "scanProgress")]
    ScanProgress(ScanProgressPayload),
    #[serde(rename = "scanLog")]
    ScanLog(ScanLogPayload),
    #[serde(rename = "scanSummary")]
    ScanSummary(ScanSummaryPayload),
    #[serde(rename = "scanPhaseTiming")]
    ScanPhaseTiming(ScanPhaseTimingPayload),
    #[serde(rename = "scanEnrichResult")]
    ScanEnrichResult(EnrichResultPayload),
    #[serde(rename = "scanDedup")]
    ScanDedup(DedupResultPayload),
    #[serde(rename = "extension-connection-status")]
    ExtensionConnectionStatus(ExtensionConnectionPayload),
    #[serde(rename = "imageQueueWorkerStarted")]
    ImageQueueWorkerStarted(ImageQueueWorkerStatusPayload),
    #[serde(rename = "imageQueueWorkerFinished")]
    ImageQueueWorkerFinished(ImageQueueWorkerStatusPayload),
    #[serde(rename = "imageQueueItemStarted")]
    ImageQueueItemStarted(ImageQueueItemPayload),
    #[serde(rename = "imageQueueItemSucceeded")]
    ImageQueueItemSucceeded(ImageQueueItemPayload),
    #[serde(rename = "imageQueueItemFailed")]
    ImageQueueItemFailed(ImageQueueItemErrorPayload),
    #[serde(rename = "appSignal")]
    AppSignal(AppSignalPayload),
    #[serde(rename = "appSignal:showMessage")]
    AppSignalShowMessage(AppSignalPayload),
    #[serde(rename = "appSignal:showErrorMessage")]
    AppSignalShowErrorMessage(AppSignalPayload),
}

impl PubSubEvent {
    pub fn event_name(&self) -> &'static str {
        match self {
            PubSubEvent::Progress(..) => "progress",
            PubSubEvent::ProgressLive(..) => "progresslive",
            PubSubEvent::ScanProgress(..) => "scanProgress",
            PubSubEvent::ScanLog(..) => "scanLog",
            PubSubEvent::ScanSummary(..) => "scanSummary",
            PubSubEvent::ScanPhaseTiming(..) => "scanPhaseTiming",
            PubSubEvent::ScanEnrichResult(..) => "scanEnrichResult",
            PubSubEvent::ScanDedup(..) => "scanDedup",
            PubSubEvent::ExtensionConnectionStatus(..) => "extension-connection-status",
            PubSubEvent::ImageQueueWorkerStarted(..) => "imageQueueWorkerStarted",
            PubSubEvent::ImageQueueWorkerFinished(..) => "imageQueueWorkerFinished",
            PubSubEvent::ImageQueueItemStarted(..) => "imageQueueItemStarted",
            PubSubEvent::ImageQueueItemSucceeded(..) => "imageQueueItemSucceeded",
            PubSubEvent::ImageQueueItemFailed(..) => "imageQueueItemFailed",
            PubSubEvent::AppSignal(..) => "appSignal",
            PubSubEvent::AppSignalShowMessage(..) => "appSignal:showMessage",
            PubSubEvent::AppSignalShowErrorMessage(..) => "appSignal:showErrorMessage",
        }
    }
}
