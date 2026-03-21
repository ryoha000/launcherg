use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::models::work_details::WorkDetailsVm;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn get_work_details_all(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<WorkDetailsVm>, CommandError> {
    let rows = modules.work_use_case().list_all_details().await?;
    Ok(rows.into_iter().map(|w| w.into()).collect())
}

#[tauri::command]
pub async fn get_work_details_by_work_id(
    modules: State<'_, Arc<Modules>>,
    work_id: String,
) -> anyhow::Result<Option<WorkDetailsVm>, CommandError> {
    let row = modules
        .work_use_case()
        .find_details_by_work_id(work_id)
        .await?;
    Ok(row.map(|w| w.into()))
}
