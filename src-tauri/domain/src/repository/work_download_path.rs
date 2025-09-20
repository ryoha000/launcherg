use crate::{work_download_path::WorkDownloadPath, works::Work, Id};
use anyhow::Result;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkDownloadPathRepository {
    async fn add(&mut self, work_id: Id<Work>, download_path: &str) -> Result<()>;
    async fn list_by_work(&mut self, work_id: Id<Work>) -> Result<Vec<WorkDownloadPath>>;
    async fn latest_by_work(&mut self, work_id: Id<Work>) -> Result<Option<WorkDownloadPath>>;
}
