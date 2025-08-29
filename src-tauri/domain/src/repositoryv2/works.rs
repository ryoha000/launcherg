use anyhow::Result;
use crate::{works::{Work, NewWork, WorkDetails}, Id};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkRepository {
    async fn upsert(&mut self, new_work: &NewWork) -> Result<Id<Work>>;
    async fn find_by_title(&mut self, title: &str) -> Result<Option<Work>>;
    async fn list_all_details(&mut self) -> Result<Vec<WorkDetails>>;
}
