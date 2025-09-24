use std::sync::Arc;
use tauri::{AppHandle, State};

use crate::domain::Id;
use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn upload_image(
    _handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    id: i32,
    base64_image: String,
) -> anyhow::Result<String, CommandError> {
    Ok(modules
        .file_use_case()
        .upload_image(id, base64_image)
        .await?)
}

#[tauri::command]
pub async fn update_collection_element_icon(
    _handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    id: i32,
    path: String,
) -> anyhow::Result<(), CommandError> {
    Ok(modules
        .image_use_case()
        .overwrite_icon_png(&Id::new(id), &path)
        .await?)
}

#[tauri::command]
pub async fn save_screenshot_by_pid(
    _handle: AppHandle,
    modules: State<'_, Arc<Modules>>,
    work_id: i32,
    process_id: u32,
) -> anyhow::Result<String, CommandError> {
    let upload_path = modules.file_use_case().get_new_upload_image_path(work_id)?;
    modules
        .process_use_case()
        .save_screenshot_by_pid(process_id, &upload_path)
        .await?;
    Ok(upload_path)
}
