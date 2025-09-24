use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};
use domain::native_host_log::{HostLogLevel, HostLogType};

#[derive(serde::Deserialize)]
pub struct GetHostLogsRequest {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
    pub level: Option<i32>,
    pub typ: Option<i32>,
}

#[derive(serde::Serialize)]
pub struct HostLogDto {
    pub id: i32,
    pub level: i32,
    pub typ: i32,
    pub message: String,
    pub created_at: String,
}

#[derive(serde::Serialize)]
pub struct HostLogsResponse {
    pub items: Vec<HostLogDto>,
    pub total: i64,
}

#[tauri::command]
pub async fn get_native_host_logs(
    modules: State<'_, Arc<Modules>>,
    request: GetHostLogsRequest,
) -> anyhow::Result<HostLogsResponse, CommandError> {
    let limit = request.limit.unwrap_or(50) as i64;
    let offset = request.offset.unwrap_or(0) as i64;
    let level = match request.level {
        Some(0) => Some(HostLogLevel::Debug),
        Some(1) => Some(HostLogLevel::Info),
        Some(2) => Some(HostLogLevel::Warn),
        Some(3) => Some(HostLogLevel::Error),
        _ => None,
    };
    let typ = match request.typ {
        Some(0) => Some(HostLogType::Unknown),
        Some(1) => Some(HostLogType::ReceiveDmmSyncGamesRequest),
        Some(2) => Some(HostLogType::ReceiveDlsiteSyncGamesRequest),
        Some(10) => Some(HostLogType::ImageQueueWorkerStarted),
        Some(11) => Some(HostLogType::ImageQueueWorkerFinished),
        Some(20) => Some(HostLogType::ImageQueueItemStarted),
        Some(21) => Some(HostLogType::ImageQueueItemSucceeded),
        Some(22) => Some(HostLogType::ImageQueueItemFailed),
        Some(30) => Some(HostLogType::ReceiveRequest),
        Some(31) => Some(HostLogType::Response),
        Some(32) => Some(HostLogType::EndProcessImageQueue),
        Some(33) => Some(HostLogType::AppSignalDispatchFailed),
        _ => None,
    };

    let items = modules
        .host_log_use_case()
        .list_logs(limit, offset, level, typ)
        .await?;
    let total = modules.host_log_use_case().count_logs(level, typ).await?;

    Ok(HostLogsResponse {
        items: items
            .into_iter()
            .map(|row| HostLogDto {
                id: row.id.value,
                level: row.level as i32,
                typ: row.r#type as i32,
                message: row.message,
                created_at: row.created_at.to_rfc3339(),
            })
            .collect(),
        total,
    })
}
