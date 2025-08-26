use anyhow::Result;
use crate::{works::{DmmWork, DlsiteWork, NewDmmWork, NewDlsiteWork, Work, NewWork, WorkDetails}, Id};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DmmWorkRepository {
    async fn upsert(&self, new_work: &NewDmmWork) -> Result<Id<DmmWork>>;
    async fn find_by_store_key(&self, store_id: &str, category: &str, subcategory: &str) -> Result<Option<DmmWork>>;
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DlsiteWorkRepository {
    async fn upsert(&self, new_work: &NewDlsiteWork) -> Result<Id<DlsiteWork>>;
    async fn find_by_store_key(&self, store_id: &str, category: &str) -> Result<Option<DlsiteWork>>;
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkRepository {
    async fn upsert(&self, new_work: &NewWork) -> Result<Id<Work>>;
    async fn find_by_title(&self, title: &str) -> Result<Option<Work>>;
    async fn list_all_details(&self) -> Result<Vec<WorkDetails>>;
}


