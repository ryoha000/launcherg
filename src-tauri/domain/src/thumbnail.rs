use crate::{works::Work, Id};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait ThumbnailService {
    async fn save_thumbnail(&self, id: &Id<Work>, url: &str) -> anyhow::Result<()>;
    async fn get_thumbnail_size(
        &self,
        id: &Id<Work>,
    ) -> anyhow::Result<Option<(u32, u32)>>;
}
