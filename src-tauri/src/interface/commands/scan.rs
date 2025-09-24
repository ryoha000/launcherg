use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};

#[tauri::command]
pub async fn scan_start(
    modules: State<'_, Arc<Modules>>,
    roots: Vec<String>,
    use_cache: Option<bool>,
) -> anyhow::Result<Vec<String>, CommandError> {
    let roots: Vec<std::path::PathBuf> = roots
        .into_iter()
        .map(|s| std::path::PathBuf::from(s))
        .collect();
    let gamenames = modules
        .work_pipeline_use_case()
        .start(roots, use_cache.unwrap_or(false))
        .await
        .map_err(|e| CommandError::Anyhow(anyhow::anyhow!(e.to_string())))?;

    let runner = modules.image_queue_runner().clone();
    tauri::async_runtime::spawn(async move {
        let _ =
            domain::service::image_queue_drain::ImageQueueDrainService::drain_until_empty(&*runner)
                .await;
    });

    Ok(gamenames)
}


