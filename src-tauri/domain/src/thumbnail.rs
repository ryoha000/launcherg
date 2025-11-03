use crate::{works::Work, StrId};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait ThumbnailService {
    async fn save_thumbnail(&self, id: &StrId<Work>, url: &str) -> anyhow::Result<()>;
    async fn get_thumbnail_size(&self, id: &StrId<Work>) -> anyhow::Result<Option<(u32, u32)>>;
}
