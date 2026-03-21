use std::sync::Arc;

use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::models::storage_paths::StoragePathSettingsVm;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn get_storage_settings(
    modules: State<'_, Arc<Modules>>,
) -> anyhow::Result<StoragePathSettingsVm, CommandError> {
    Ok(modules.app_settings_use_case().get_storage_settings().await?.into())
}

#[tauri::command]
pub async fn set_storage_settings(
    modules: State<'_, Arc<Modules>>,
    settings: StoragePathSettingsVm,
) -> anyhow::Result<StoragePathSettingsVm, CommandError> {
    let saved = modules
        .app_settings_use_case()
        .set_storage_settings(settings.into())
        .await?;
    modules.storage_path_settings().set(saved.clone().into());
    Ok(saved.into())
}
