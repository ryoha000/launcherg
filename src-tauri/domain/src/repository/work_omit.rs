use anyhow::Result;
use crate::{work_omit::WorkOmit, Id, works::Work};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkOmitRepository {
    async fn add(&self, work_id: Id<Work>) -> Result<()>;
    async fn remove(&self, work_id: Id<Work>) -> Result<()>;
    async fn list(&self) -> Result<Vec<WorkOmit>>;
    async fn exists(&self, work_id: Id<Work>) -> Result<bool>;
}


