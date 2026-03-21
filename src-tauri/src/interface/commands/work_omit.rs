use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::models::work_omit::WorkOmitItemVm;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn work_omit_add(
    modules: State<'_, Arc<Modules>>,
    work_id: String,
) -> anyhow::Result<(), CommandError> {
    modules
        .work_omit_use_case()
        .add(domain::StrId::new(work_id))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn work_omit_remove(
    modules: State<'_, Arc<Modules>>,
    work_id: String,
) -> anyhow::Result<(), CommandError> {
    modules
        .work_omit_use_case()
        .remove(domain::StrId::new(work_id))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn work_omit_all(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<WorkOmitItemVm>, CommandError> {
    let list = modules.work_omit_use_case().list().await?;
    Ok(list.into_iter().map(|e| e.into()).collect())
}
