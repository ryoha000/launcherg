use crate::{
    works::{DlsiteWork, DmmWork, NewDlsiteWork, NewDmmWork, NewWork, Work, WorkDetails},
    Id, StrId,
};
use anyhow::Result;
use chrono::{DateTime, Local};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait WorkRepository {
    async fn upsert(&mut self, new_work: &NewWork) -> Result<StrId<Work>>;
    async fn find_by_title(&mut self, title: &str) -> Result<Option<Work>>;
    async fn list_all_details(&mut self) -> Result<Vec<WorkDetails>>;
    async fn find_details_by_work_id(&mut self, work_id: StrId<Work>) -> Result<Option<WorkDetails>>;
    async fn delete(&mut self, id: StrId<Work>) -> Result<()>;
    async fn find_work_ids_by_erogamescape_ids(
        &mut self,
        erogamescape_ids: &[i32],
    ) -> Result<Vec<(i32, StrId<Work>)>>;
    async fn upsert_erogamescape_map(
        &mut self,
        work_id: StrId<Work>,
        erogamescape_id: i32,
    ) -> Result<()>;
    async fn list_work_ids_missing_thumbnail_size(&mut self) -> Result<Vec<StrId<Work>>>;
    async fn upsert_work_thumbnail_size(
        &mut self,
        work_id: StrId<Work>,
        width: i32,
        height: i32,
    ) -> Result<()>;
    async fn update_last_play_at_by_work_id(
        &mut self,
        work_id: StrId<Work>,
        last_play_at: DateTime<Local>,
    ) -> Result<()>;
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DmmWorkRepository {
    async fn upsert(&mut self, new_work: &NewDmmWork) -> Result<Id<DmmWork>>;
    async fn find_by_store_key(
        &mut self,
        store_id: &str,
        category: &str,
        subcategory: &str,
    ) -> Result<Option<DmmWork>>;
    async fn find_by_store_id(&mut self, store_id: &str) -> Result<Option<DmmWork>>;
    async fn find_by_store_keys(
        &mut self,
        keys: &[(String, String, String)],
    ) -> Result<Vec<DmmWork>>;
    async fn find_by_work_id(&mut self, work_id: StrId<Work>) -> Result<Option<DmmWork>>;
}

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait DlsiteWorkRepository {
    async fn upsert(&mut self, new_work: &NewDlsiteWork) -> Result<Id<DlsiteWork>>;
    async fn find_by_store_key(
        &mut self,
        store_id: &str,
        category: &str,
    ) -> Result<Option<DlsiteWork>>;
    async fn find_by_store_id(&mut self, store_id: &str) -> Result<Option<DlsiteWork>>;
}
