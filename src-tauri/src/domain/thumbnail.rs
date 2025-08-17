use async_trait::async_trait;

use crate::domain::{collection::CollectionElement, Id};

#[async_trait]
pub trait ThumbnailService: Send + Sync {
    async fn save_thumbnail(&self, id: &Id<CollectionElement>, url: &str) -> anyhow::Result<()>;
}
