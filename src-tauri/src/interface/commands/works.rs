use std::sync::Arc;
use tauri::State;
use chrono::Utc;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};
use crate::interface::models::all_game_cache::AllGameCacheOne;
use crate::interface::models::work_path_input::WorkPathInput;
use domain::pubsub::event::{AppSignalEventPayload, AppSignalPayload, AppSignalSourcePayload, PubSubEvent};
use domain::pubsub::PubSubService;
use domain::service::work_registration::RegisterWorkPath;

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
    work_id: String,
) -> anyhow::Result<Vec<(i32, String)>, CommandError> {
    Ok(modules.work_use_case().list_work_lnks(work_id).await?)
}

#[tauri::command]
pub async fn get_work_paths(
    modules: State<'_, Arc<Modules>>,
    work_id: String,
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
    work_id: String,
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
pub async fn update_work_like(
    modules: State<'_, Arc<Modules>>,
    work_id: String,
    is_like: bool,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .work_use_case()
        .update_like(work_id, is_like)
        .await?)
}

#[tauri::command]
pub async fn register_work_from_path(
    modules: State<'_, Arc<Modules>>,
    path: WorkPathInput,
    game_cache: AllGameCacheOne,
) -> anyhow::Result<(), CommandError> {
    let input = match path {
        WorkPathInput::Exe { exe_path } => RegisterWorkPath::Exe { exe_path },
        WorkPathInput::Lnk { lnk_path } => RegisterWorkPath::Lnk { lnk_path },
    };

    Ok(modules
        .work_use_case()
        .register_work_from_input(
            game_cache.id,
            game_cache.gamename,
            game_cache.thumbnail_url,
            input,
        )
        .await?)
}

#[tauri::command]
pub async fn process_pending_exe_links(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules.work_link_pending_exe_use_case().process_pending_exe_links().await?)
}
