use crate::{work_download_path::WorkDownloadPath, works::Work, StrId};
use anyhow::Result;

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkDownloadPathRepository {
    async fn add(&mut self, work_id: StrId<Work>, download_path: &str) -> Result<()>;
    async fn list_by_work(&mut self, work_id: StrId<Work>) -> Result<Vec<WorkDownloadPath>>;
    async fn latest_by_work(&mut self, work_id: StrId<Work>) -> Result<Option<WorkDownloadPath>>;
}
