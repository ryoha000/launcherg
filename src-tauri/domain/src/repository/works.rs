use anyhow::Result;
use crate::{works::{Work, NewWork, WorkDetails, DmmWork, NewDmmWork, DlsiteWork, NewDlsiteWork}, Id};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkRepository {
    async fn upsert(&mut self, new_work: &NewWork) -> Result<Id<Work>>;
    async fn find_by_title(&mut self, title: &str) -> Result<Option<Work>>;
    async fn list_all_details(&mut self) -> Result<Vec<WorkDetails>>;
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DmmWorkRepository {
    async fn upsert(&mut self, new_work: &NewDmmWork) -> Result<Id<DmmWork>>;
    async fn find_by_store_key(&mut self, store_id: &str, category: &str, subcategory: &str) -> Result<Option<DmmWork>>;
    async fn find_by_store_keys(&mut self, keys: &[(String, String, String)]) -> Result<Vec<DmmWork>>;
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DlsiteWorkRepository {
    async fn upsert(&mut self, new_work: &NewDlsiteWork) -> Result<Id<DlsiteWork>>;
    async fn find_by_store_key(&mut self, store_id: &str, category: &str) -> Result<Option<DlsiteWork>>;
}
