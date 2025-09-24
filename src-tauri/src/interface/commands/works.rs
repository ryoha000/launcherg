use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn list_work_lnks(
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
) -> anyhow::Result<Vec<(i32, String)>, CommandError> {
    Ok(modules.work_use_case().list_work_lnks(work_id).await?)
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


