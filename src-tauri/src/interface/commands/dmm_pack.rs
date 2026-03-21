use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn work_pack_add(
    modules: State<'_, Arc<Modules>>,
    work_id: String,
) -> anyhow::Result<(), CommandError> {
    modules
        .dmm_pack_use_case()
        .add(domain::StrId::new(work_id))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn work_pack_remove(
    modules: State<'_, Arc<Modules>>,
    work_id: String,
) -> anyhow::Result<(), CommandError> {
    modules
        .dmm_pack_use_case()
        .remove(domain::StrId::new(work_id))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn work_pack_all(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<String>, CommandError> {
    let list = modules.dmm_pack_use_case().list().await?;
    Ok(list.into_iter().map(|e| e.work_id.value).collect())
}
