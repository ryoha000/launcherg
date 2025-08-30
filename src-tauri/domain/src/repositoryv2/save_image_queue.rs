use crate::{save_image_queue::{ImageSaveQueueRow, ImageSrcType, ImagePreprocess}, Id};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait ImageSaveQueueRepository {
    async fn enqueue(&mut self, src: &str, src_type: ImageSrcType, dst_path: &str, preprocess: ImagePreprocess) -> anyhow::Result<Id<ImageSaveQueueRow>>;
    async fn list_unfinished_oldest(&mut self, limit: i64) -> anyhow::Result<Vec<ImageSaveQueueRow>>;
    async fn mark_finished(&mut self, id: Id<ImageSaveQueueRow>) -> anyhow::Result<()>;
    async fn mark_failed(&mut self, id: Id<ImageSaveQueueRow>, error: &str) -> anyhow::Result<()>;
}


