use crate::{works::NewWorkLike, works::Work, works::WorkLike, Id};
use anyhow::Result;
use chrono::{DateTime, Local};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkLikeRepository {
    async fn upsert(&mut self, like: &NewWorkLike) -> Result<Id<WorkLike>>;
    async fn delete_by_work_id(&mut self, work_id: Id<Work>) -> Result<()>;
    async fn get_by_work_id(&mut self, work_id: Id<Work>) -> Result<Option<WorkLike>>;
    async fn update_like_at_by_work_id(
        &mut self,
        work_id: Id<Work>,
        like_at: Option<DateTime<Local>>,
    ) -> Result<()>;
}
