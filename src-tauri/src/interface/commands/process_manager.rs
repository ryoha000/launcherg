use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};
use domain::windows::proctail_manager::{ProcTailManagerStatus, ProcTailVersion};

#[tauri::command]
pub async fn proctail_manager_get_status(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<ProcTailManagerStatus, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_get_status()
        .await?)
}

#[tauri::command]
pub async fn proctail_manager_get_latest_version(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<ProcTailVersion, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_get_latest_version()
        .await?)
}

#[tauri::command]
pub async fn proctail_manager_is_update_available(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<bool, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_is_update_available()
        .await?)
}

#[tauri::command]
pub async fn proctail_manager_download_and_install(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_download_and_install()
        .await?)
}

#[tauri::command]
pub async fn proctail_manager_start(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules.process_use_case().proctail_manager_start().await?)
}

#[tauri::command]
pub async fn proctail_manager_stop(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<(), CommandError> {
    Ok(modules.process_use_case().proctail_manager_stop().await?)
}

#[tauri::command]
pub async fn proctail_manager_is_running(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<bool, CommandError> {
    Ok(modules
        .process_use_case()
        .proctail_manager_is_running()
        .await?)
}


