use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::models::parent_dmm_pack::DmmPackKeysVm;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn work_pack_add(
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
) -> anyhow::Result<(), CommandError> {
    modules
        .dmm_pack_use_case()
        .add(domain::Id::new(work_id))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn work_pack_remove(
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
) -> anyhow::Result<(), CommandError> {
    modules
        .dmm_pack_use_case()
        .remove(domain::Id::new(work_id))
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn work_pack_all(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<Vec<i32>, CommandError> {
    let list = modules.dmm_pack_use_case().list().await?;
    Ok(list.into_iter().map(|e| e.work_id.value).collect())
}

#[tauri::command]
pub async fn get_parent_dmm_pack_keys(
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
) -> anyhow::Result<Option<DmmPackKeysVm>, CommandError> {
    let parent_id = modules
        .work_use_case()
        .get_parent_dmm_pack_work_id(work_id)
        .await?;
    if let Some(pid) = parent_id {
        if let Some(dmm) = modules.work_use_case().get_dmm_work_by_work_id(pid).await? {
            return Ok(Some(DmmPackKeysVm {
                store_id: dmm.store_id,
                category: dmm.category,
                subcategory: dmm.subcategory,
            }));
        }
    }
    Ok(None)
}


