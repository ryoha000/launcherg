use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};
use chrono::Utc;
use domain::pubsub::PubSubService as _;
use domain::pubsub::event::{AppSignalEventPayload, AppSignalPayload, AppSignalSourcePayload, PubSubEvent};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkLnkVm {
    pub id: i32,
    pub lnk_path: String,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkPathsVm {
    pub lnks: Vec<WorkLnkVm>,
}

#[tauri::command]
pub async fn backfill_thumbnail_sizes(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<usize, CommandError> {
    let updated = modules
        .work_pipeline_use_case()
        .backfill_thumbnail_sizes()
        .await?;
    if updated > 0 {
        let payload = AppSignalPayload {
            source: AppSignalSourcePayload::NativeMessagingHost,
            event: AppSignalEventPayload::RefetchWorks,
            issued_at: Utc::now(),
        };
        modules.pubsub().notify(PubSubEvent::AppSignalRefetchWorks(payload))?;
    }
    Ok(updated)
}

#[tauri::command]
pub async fn list_work_lnks(
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
) -> anyhow::Result<Vec<(i32, String)>, CommandError> {
    Ok(modules.work_use_case().list_work_lnks(work_id).await?)
}

#[tauri::command]
pub async fn get_work_paths(
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
) -> anyhow::Result<WorkPathsVm, CommandError> {
    let list = modules.work_use_case().list_work_lnks(work_id).await?;
    let lnks = list
        .into_iter()
        .map(|(id, path)| WorkLnkVm { id, lnk_path: path })
        .collect();
    Ok(WorkPathsVm { lnks })
}

#[tauri::command]
pub async fn delete_work(
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
) -> anyhow::Result<(), CommandError> {
    Ok(modules.work_use_case().delete_work(work_id).await?)
}

#[tauri::command]
pub async fn launch_work(
    modules: State<'_, Arc<Modules>>,
    is_run_as_admin: bool,
    work_lnk_id: i32,
) -> anyhow::Result<Option<u32>, CommandError> {
    Ok(modules
        .work_use_case()
        .launch_work(is_run_as_admin, work_lnk_id)
        .await?)
}

#[tauri::command]
pub async fn migrate_collection_paths_to_work_lnks(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .collection_use_case()
        .migrate_collection_paths_to_work_lnks()
        .await?)
}

#[tauri::command]
pub async fn update_work_like(
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
    is_like: bool,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .work_use_case()
        .update_like(work_id, is_like)
        .await?)
}
