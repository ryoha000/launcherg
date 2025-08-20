use crate::{collection::CollectionElement, Id};

#[trait_variant::make(Send)]
pub trait ThumbnailService {
    async fn save_thumbnail(&self, id: &Id<CollectionElement>, url: &str) -> anyhow::Result<()>;
    async fn get_thumbnail_size(&self, id: &Id<CollectionElement>) -> anyhow::Result<Option<(u32, u32)>>;
}
