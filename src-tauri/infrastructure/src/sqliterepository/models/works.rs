#[derive(sqlx::FromRow, Clone)]
pub struct DmmWorkTable {
    pub id: i64,
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
    pub work_id: i64,
}

#[derive(sqlx::FromRow, Clone)]
pub struct DlsiteWorkTable {
    pub id: i64,
    pub store_id: String,
    pub category: String,
    pub work_id: i64,
}

#[derive(sqlx::FromRow, Clone)]
pub struct WorkTable {
    pub id: i64,
    pub title: String,
}

#[derive(sqlx::FromRow, Clone)]
pub struct WorkDetailsRow {
    pub work_id: i64,
    pub work_title: String,
    pub dmm_id: Option<i64>,
    pub dmm_store_id: Option<String>,
    pub dmm_category: Option<String>,
    pub dmm_subcategory: Option<String>,
    pub ce_id: Option<i64>,
    pub egs_id: Option<i64>,
    pub egs_erogamescape_id: Option<i32>,
    pub egs_created_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub egs_updated_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub omit_id: Option<i64>,
    pub dmm_pack_id: Option<i64>,
    pub dlsite_id: Option<i64>,
    pub dlsite_store_id: Option<String>,
    pub dlsite_category: Option<String>,
    pub latest_path_id: Option<i64>,
    pub latest_path_download_path: Option<String>,
    pub like_id: Option<i64>,
    pub like_like_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub like_created_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub like_updated_at: Option<sqlx::types::chrono::NaiveDateTime>,
}

impl From<crate::sqliterepository::models::works::WorkDetailsRow> for domain::works::WorkDetails {
    fn from(r: crate::sqliterepository::models::works::WorkDetailsRow) -> Self {
        use domain::{
            collection::CollectionElementErogamescape,
            works::{DlsiteWork, DmmWork, Work},
            Id,
        };
        let mut details = domain::works::WorkDetails {
            work: Work {
                id: Id::new(r.work_id as i32),
                title: r.work_title.clone(),
            },
            dmm: None,
            dlsite: None,
            collection_element_id: r.ce_id.map(|v| Id::new(v as i32)),
            erogamescape: None,
            is_omitted: false,
            is_dmm_pack: false,
            latest_download_path: None,
            like: None,
        };

        if let Some(dmm_id) = r.dmm_id {
            details.dmm = Some(DmmWork {
                id: Id::new(dmm_id as i32),
                work_id: Id::new(r.work_id as i32),
                store_id: r.dmm_store_id.unwrap_or_default(),
                category: r.dmm_category.unwrap_or_default(),
                subcategory: r.dmm_subcategory.unwrap_or_default(),
            });
            details.is_dmm_pack = r.dmm_pack_id.is_some();
        }

        if let Some(path_id) = r.latest_path_id {
            if let Some(download_path) = r.latest_path_download_path.clone() {
                details.latest_download_path = Some(domain::work_download_path::WorkDownloadPath {
                    id: Id::new(path_id as i32),
                    work_id: Id::new(r.work_id as i32),
                    download_path,
                });
            }
        }

        if let Some(dl_id) = r.dlsite_id {
            details.dlsite = Some(DlsiteWork {
                id: Id::new(dl_id as i32),
                work_id: Id::new(r.work_id as i32),
                store_id: r.dlsite_store_id.unwrap_or_default(),
                category: r.dlsite_category.unwrap_or_default(),
            });
        }

        if let Some(_) = r.omit_id {
            details.is_omitted = true;
        }

        if let Some(egs_row_id) = r.egs_id {
            if let (Some(egs_id), Some(created), Some(updated), Some(ce_id)) = (
                r.egs_erogamescape_id,
                r.egs_created_at,
                r.egs_updated_at,
                r.ce_id,
            ) {
                details.erogamescape = Some(CollectionElementErogamescape::new(
                    Id::new(egs_row_id as i32),
                    Id::new(ce_id as i32),
                    egs_id,
                    created.and_utc().with_timezone(&chrono::Local),
                    updated.and_utc().with_timezone(&chrono::Local),
                ));
            }
        }

        if let Some(like_id) = r.like_id {
            if let (Some(like_at), Some(created), Some(updated)) =
                (r.like_like_at, r.like_created_at, r.like_updated_at)
            {
                details.like = Some(domain::works::WorkLike {
                    id: Id::new(like_id as i32),
                    work_id: Id::new(r.work_id as i32),
                    like_at: like_at.and_utc().with_timezone(&chrono::Local),
                    created_at: created.and_utc().with_timezone(&chrono::Local),
                    updated_at: updated.and_utc().with_timezone(&chrono::Local),
                });
            }
        }

        details
    }
}

impl TryFrom<crate::sqliterepository::models::works::DmmWorkTable> for domain::works::DmmWork {
    type Error = anyhow::Error;
    fn try_from(
        v: crate::sqliterepository::models::works::DmmWorkTable,
    ) -> Result<Self, Self::Error> {
        Ok(domain::works::DmmWork {
            id: domain::Id::new(v.id as i32),
            work_id: domain::Id::new(v.work_id as i32),
            store_id: v.store_id,
            category: v.category,
            subcategory: v.subcategory,
        })
    }
}

impl TryFrom<crate::sqliterepository::models::works::DlsiteWorkTable>
    for domain::works::DlsiteWork
{
    type Error = anyhow::Error;
    fn try_from(
        v: crate::sqliterepository::models::works::DlsiteWorkTable,
    ) -> Result<Self, Self::Error> {
        Ok(domain::works::DlsiteWork {
            id: domain::Id::new(v.id as i32),
            work_id: domain::Id::new(v.work_id as i32),
            store_id: v.store_id,
            category: v.category,
        })
    }
}

impl TryFrom<crate::sqliterepository::models::works::WorkTable> for domain::works::Work {
    type Error = anyhow::Error;
    fn try_from(v: crate::sqliterepository::models::works::WorkTable) -> Result<Self, Self::Error> {
        Ok(domain::works::Work {
            id: domain::Id::new(v.id as i32),
            title: v.title,
        })
    }
}

#[derive(sqlx::FromRow, Clone)]
pub struct WorkLikeRow {
    pub id: i64,
    pub work_id: i64,
    pub like_at: sqlx::types::chrono::NaiveDateTime,
    pub created_at: sqlx::types::chrono::NaiveDateTime,
    pub updated_at: sqlx::types::chrono::NaiveDateTime,
}

impl TryFrom<crate::sqliterepository::models::works::WorkLikeRow> for domain::works::WorkLike {
    type Error = anyhow::Error;
    fn try_from(
        v: crate::sqliterepository::models::works::WorkLikeRow,
    ) -> Result<Self, Self::Error> {
        Ok(domain::works::WorkLike {
            id: domain::Id::new(v.id as i32),
            work_id: domain::Id::new(v.work_id as i32),
            like_at: v.like_at.and_utc().with_timezone(&chrono::Local),
            created_at: v.created_at.and_utc().with_timezone(&chrono::Local),
            updated_at: v.updated_at.and_utc().with_timezone(&chrono::Local),
        })
    }
}

#[derive(sqlx::FromRow, Clone)]
pub struct WorkLnkRow {
    pub id: i64,
    pub work_id: i64,
    pub lnk_path: String,
}

impl From<crate::sqliterepository::models::works::WorkLnkRow>
    for domain::repository::work_lnk::WorkLnk
{
    fn from(v: crate::sqliterepository::models::works::WorkLnkRow) -> Self {
        domain::repository::work_lnk::WorkLnk {
            id: domain::Id::new(v.id as i32),
            work_id: domain::Id::new(v.work_id as i32),
            lnk_path: v.lnk_path,
        }
    }
}
