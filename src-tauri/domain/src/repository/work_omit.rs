use crate::{work_omit::WorkOmit, works::Work, Id};
use anyhow::Result;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkOmitRepository {
    async fn add(&mut self, work_id: Id<Work>) -> Result<()>;
    async fn remove(&mut self, work_id: Id<Work>) -> Result<()>;
    async fn list(&mut self) -> Result<Vec<WorkOmit>>;
    async fn exists(&mut self, work_id: Id<Work>) -> Result<bool>;
}
