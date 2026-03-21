use crate::{work_omit::WorkOmit, works::Work, StrId};
use anyhow::Result;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkOmitRepository {
    async fn add(&mut self, work_id: StrId<Work>) -> Result<()>;
    async fn remove(&mut self, work_id: StrId<Work>) -> Result<()>;
    async fn list(&mut self) -> Result<Vec<WorkOmit>>;
    async fn exists(&mut self, work_id: StrId<Work>) -> Result<bool>;
}
