use async_trait::async_trait;

use crate::domain::{collection::CollectionElement, Id};

#[async_trait]
pub trait IconService: Send + Sync {
    async fn save_icon_from_path(
        &self,
        id: &Id<CollectionElement>,
        source_path: &str,
    ) -> anyhow::Result<()>;

    async fn save_default_icon(&self, id: &Id<CollectionElement>) -> anyhow::Result<()>;
}


