use crate::domain::Id;
use chrono::{DateTime, Local};

#[derive(Debug, Clone, Copy)]
pub enum HostLogLevel { Info = 1, Warn = 2, Error = 3 }

#[derive(Debug, Clone, Copy)]
pub enum HostLogType {
    Unknown = 0,
    ReceiveDmmSyncGamesRequest = 1,
    ReceiveDlsiteSyncGamesRequest = 2,
    ImageQueueWorkerStarted = 10,
    ImageQueueWorkerFinished = 11,
    ImageQueueItemStarted = 20,
    ImageQueueItemSucceeded = 21,
    ImageQueueItemFailed = 22,
}

#[derive(Debug, Clone)]
pub struct NativeHostLogRow {
    pub id: Id<NativeHostLogRow>,
    pub level: HostLogLevel,
    pub r#type: HostLogType,
    pub message: String,
    pub created_at: DateTime<Local>,
}


