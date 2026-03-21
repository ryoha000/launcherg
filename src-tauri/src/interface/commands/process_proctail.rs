use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};
use domain::windows::proctail::{HealthCheckResult, ProcTailEvent, ServiceStatus, WatchTarget};

#[derive(serde::Deserialize)]
pub struct AddWatchTargetRequest {
    #[serde(rename = "processId")]
    pub process_id: u32,
    pub tag: String,
}

#[tauri::command]
pub async fn proctail_add_watch_target(
    modules: State<'_, Arc<Modules>>,
    request: AddWatchTargetRequest,
) -> anyhow::Result<WatchTarget, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_add_watch_target(request.process_id, &request.tag)
        .await?)
}

#[derive(serde::Deserialize)]
pub struct RemoveWatchTargetRequest {
    pub tag: String,
}

#[tauri::command]
pub async fn proctail_remove_watch_target(
    modules: State<'_, Arc<Modules>>,
    request: RemoveWatchTargetRequest,
) -> anyhow::Result<u32, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_remove_watch_target(&request.tag)
        .await?)
}

#[tauri::command]
pub async fn proctail_get_watch_targets(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<WatchTarget>, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_get_watch_targets()
        .await?)
}

#[derive(serde::Deserialize)]
pub struct GetEventsRequest {
    pub tag: String,
    pub count: Option<u32>,
    #[serde(rename = "eventType")]
    pub event_type: Option<String>,
}

#[tauri::command]
pub async fn proctail_get_recorded_events(
    modules: State<'_, Arc<Modules>>,
    request: GetEventsRequest,
) -> anyhow::Result<Vec<ProcTailEvent>, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_get_recorded_events(&request.tag, request.count, request.event_type.as_deref())
        .await?)
}

#[derive(serde::Deserialize)]
pub struct ClearEventsRequest {
    pub tag: String,
}

#[tauri::command]
pub async fn proctail_clear_events(
    modules: State<'_, Arc<Modules>>,
    request: ClearEventsRequest,
) -> anyhow::Result<u32, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_clear_events(&request.tag)
        .await?)
}

#[tauri::command]
pub async fn proctail_get_status(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<ServiceStatus, CommandError> {
    Ok(modules.process_use_case().proctail_get_status().await?)
}

#[tauri::command]
pub async fn proctail_health_check(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<HealthCheckResult, CommandError> {
    Ok(modules.process_use_case().proctail_health_check().await?)
}

#[tauri::command]
pub async fn proctail_is_service_available(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<bool, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_is_service_available()
        .await?)
}
