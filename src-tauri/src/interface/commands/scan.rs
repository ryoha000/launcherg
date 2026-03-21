use chrono::Utc;
use std::sync::Arc;
use tauri::State;

use crate::interface::error::CommandError;
use crate::interface::module::{Modules, ModulesExt};
use domain::pubsub::event::{
    AppSignalEventPayload, AppSignalPayload, AppSignalSourcePayload, PubSubEvent,
};
use domain::pubsub::PubSubService;
use domain::service::image_queue_drain::ImageQueueDrainService;

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
    // 1. Work をスキャン・登録
    let gamenames = modules
        .work_pipeline_use_case()
        .start(roots, use_cache.unwrap_or(false))
        .await
        .map_err(|e| CommandError::Anyhow(anyhow::anyhow!(e.to_string())))?;

    // 2. ImageQueue の完了を待機
    let runner = modules.image_queue_runner();
    ImageQueueDrainService::drain_until_empty(runner.as_ref()).await?;

    // 3. サムネイルサイズを画像生成後に再取得
    modules
        .work_thumbnail_use_case()
        .backfill_thumbnail_sizes()
        .await
        .map_err(|e| CommandError::Anyhow(anyhow::anyhow!(e.to_string())))?;

    // 4. RefetchWorks を通知
    let payload = AppSignalPayload {
        source: AppSignalSourcePayload::NativeMessagingHost,
        event: AppSignalEventPayload::RefetchWorks,
        issued_at: Utc::now(),
    };
    modules
        .pubsub()
        .notify(PubSubEvent::AppSignalRefetchWorks(payload))?;

    Ok(gamenames)
}
