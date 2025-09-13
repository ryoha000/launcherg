use crate::save_image_queue::ImageSaveQueueRow;
use futures::future::BoxFuture;

#[mockall::automock]
pub trait ImageQueueWorkerEventHandler: Send + Sync {
    fn on_worker_started(&self) -> BoxFuture<'static, anyhow::Result<()>>;
    fn on_worker_finished(&self) -> BoxFuture<'static, anyhow::Result<()>>;
    fn on_item_started(&self, item: &ImageSaveQueueRow) -> BoxFuture<'static, anyhow::Result<()>>;
    fn on_item_succeeded(&self, item: &ImageSaveQueueRow) -> BoxFuture<'static, anyhow::Result<()>>;
    fn on_item_failed(&self, item: &ImageSaveQueueRow, error_message: &str) -> BoxFuture<'static, anyhow::Result<()>>;
}



