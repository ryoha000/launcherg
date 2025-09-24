use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::models::save_image_queue::ImageSaveQueueRowVm;
use crate::interface::module::{Modules, ModulesExt};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetImageSaveQueueRequest {
    pub limit: Option<i64>,
    pub status: Option<String>,
}

#[tauri::command]
pub async fn get_image_save_queue(
    modules: State<'_, Arc<Modules>>,
    request: Option<GetImageSaveQueueRequest>,
) -> anyhow::Result<Vec<ImageSaveQueueRowVm>, CommandError> {
    let limit = request.as_ref().and_then(|r| r.limit).unwrap_or(500);
    let status = request
        .and_then(|r| r.status)
        .unwrap_or_else(|| "unfinished".to_string());
    let rows = modules
        .image_queue_use_case()
        .list(status.as_str() == "unfinished", limit)
        .await?;
    Ok(rows.into_iter().map(|r| r.into()).collect())
}
