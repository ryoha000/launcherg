#[derive(sqlx::FromRow, Clone)]
pub struct DmmWorkTable {
    pub id: i64,
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
    pub work_id: String,
}

#[derive(sqlx::FromRow, Clone)]
pub struct DlsiteWorkTable {
    pub id: i64,
    pub store_id: String,
    pub category: String,
    pub work_id: String,
}

#[derive(sqlx::FromRow, Clone)]
pub struct WorkTable {
    pub id: String,
    pub title: String,
}

#[derive(sqlx::FromRow, Clone)]
pub struct WorkDetailsRow {
    pub work_id: String,
    pub work_title: String,
    pub ce_created_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub dmm_id: Option<i64>,
    pub dmm_store_id: Option<String>,
    pub dmm_category: Option<String>,
    pub dmm_subcategory: Option<String>,
    pub ce_id: Option<i64>,
    pub egs_id: Option<i64>,
    pub egs_erogamescape_id: Option<i32>,
    pub egs_created_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub egs_updated_at: Option<sqlx::types::chrono::NaiveDateTime>,
    // Erogamescape information details (from erogamescape_information)
    pub egs_info_gamename_ruby: Option<String>,
    pub egs_info_brandname: Option<String>,
    pub egs_info_brandname_ruby: Option<String>,
    pub egs_info_sellday: Option<String>,
    pub egs_info_is_nukige: Option<i64>,
    pub egs_info_created_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub egs_info_updated_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub cet_width: Option<i64>,
    pub cet_height: Option<i64>,
    pub omit_id: Option<i64>,
    pub dmm_pack_id: Option<i64>,
    pub dlsite_id: Option<i64>,
    pub dlsite_store_id: Option<String>,
    pub dlsite_category: Option<String>,
    pub latest_path_id: Option<i64>,
    pub latest_path_download_path: Option<String>,
    pub install_install_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub play_last_play_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub like_id: Option<i64>,
    pub like_like_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub like_created_at: Option<sqlx::types::chrono::NaiveDateTime>,
    pub like_updated_at: Option<sqlx::types::chrono::NaiveDateTime>,
}

impl From<crate::sqliterepository::models::works::WorkDetailsRow> for domain::works::WorkDetails {
    fn from(r: crate::sqliterepository::models::works::WorkDetailsRow) -> Self {
        use domain::{erogamescape::ErogamescapeInformation, works::{DlsiteWork, DmmWork, Work, WorkThumbnailSize}, Id, StrId};
        let mut details = domain::works::WorkDetails {
            work: Work {
                id: StrId::new(r.work_id.clone()),
                title: r.work_title.clone(),
            },
            dmm: None,
            dlsite: None,
            erogamescape_id: r.egs_erogamescape_id,
            erogamescape_information: None,
            is_omitted: false,
            is_dmm_pack: false,
            latest_download_path: None,
            like: None,
            install_at: r
                .install_install_at
                .map(|v| v.and_utc().with_timezone(&chrono::Local)),
            last_play_at: r
                .play_last_play_at
                .map(|v| v.and_utc().with_timezone(&chrono::Local)),
            registered_at: r
                .ce_created_at
                .map(|v| v.and_utc().with_timezone(&chrono::Local)),
            thumbnail_size: None,
        };

        if let Some(dmm_id) = r.dmm_id {
            details.dmm = Some(DmmWork {
                id: Id::new(dmm_id as i32),
                work_id: StrId::new(r.work_id.clone()),
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
                    work_id: StrId::new(r.work_id.clone()),
                    download_path,
                });
            }
        }

        if let Some(dl_id) = r.dlsite_id {
            details.dlsite = Some(DlsiteWork {
                id: Id::new(dl_id as i32),
                work_id: StrId::new(r.work_id.clone()),
                store_id: r.dlsite_store_id.unwrap_or_default(),
                category: r.dlsite_category.unwrap_or_default(),
            });
        }

        if let Some(_) = r.omit_id {
            details.is_omitted = true;
        }

        // Map erogamescape_information when available
        if let Some(info_egs_id) = r.egs_erogamescape_id {
            if let (
                Some(gamename_ruby),
                Some(brandname),
                Some(brandname_ruby),
                Some(sellday),
                Some(is_nukige),
                Some(created_at),
                Some(updated_at),
            ) = (
                r.egs_info_gamename_ruby.clone(),
                r.egs_info_brandname.clone(),
                r.egs_info_brandname_ruby.clone(),
                r.egs_info_sellday.clone(),
                r.egs_info_is_nukige,
                r.egs_info_created_at,
                r.egs_info_updated_at,
            ) {
                details.erogamescape_information = Some(ErogamescapeInformation::new(
                    Id::new(info_egs_id),
                    gamename_ruby,
                    brandname,
                    brandname_ruby,
                    sellday,
                    is_nukige != 0,
                    created_at.and_utc().with_timezone(&chrono::Local),
                    updated_at.and_utc().with_timezone(&chrono::Local),
                ));
            }
        }

        if let Some(like_id) = r.like_id {
            if let (Some(like_at), Some(created), Some(updated)) =
                (r.like_like_at, r.like_created_at, r.like_updated_at)
            {
                details.like = Some(domain::works::WorkLike {
                    id: Id::new(like_id as i32),
                    work_id: StrId::new(r.work_id.clone()),
                    like_at: like_at.and_utc().with_timezone(&chrono::Local),
                    created_at: created.and_utc().with_timezone(&chrono::Local),
                    updated_at: updated.and_utc().with_timezone(&chrono::Local),
                });
            }
        }

        if let (Some(w), Some(h)) = (r.cet_width, r.cet_height) {
            details.thumbnail_size = Some(WorkThumbnailSize::new(w as i32, h as i32))
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
            work_id: domain::StrId::new(v.work_id),
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
            work_id: domain::StrId::new(v.work_id),
            store_id: v.store_id,
            category: v.category,
        })
    }
}

impl TryFrom<crate::sqliterepository::models::works::WorkTable> for domain::works::Work {
    type Error = anyhow::Error;
    fn try_from(v: crate::sqliterepository::models::works::WorkTable) -> Result<Self, Self::Error> {
        Ok(domain::works::Work {
            id: domain::StrId::new(v.id),
            title: v.title,
        })
    }
}

#[derive(sqlx::FromRow, Clone)]
pub struct WorkLikeRow {
    pub id: i64,
    pub work_id: String,
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
            work_id: domain::StrId::new(v.work_id),
            like_at: v.like_at.and_utc().with_timezone(&chrono::Local),
            created_at: v.created_at.and_utc().with_timezone(&chrono::Local),
            updated_at: v.updated_at.and_utc().with_timezone(&chrono::Local),
        })
    }
}

#[derive(sqlx::FromRow, Clone)]
pub struct WorkLnkRow {
    pub id: i64,
    pub work_id: String,
    pub lnk_path: String,
}

impl From<crate::sqliterepository::models::works::WorkLnkRow>
    for domain::repository::work_lnk::WorkLnk
{
    fn from(v: crate::sqliterepository::models::works::WorkLnkRow) -> Self {
        domain::repository::work_lnk::WorkLnk {
            id: domain::Id::new(v.id as i32),
            work_id: domain::StrId::new(v.work_id),
            lnk_path: v.lnk_path,
        }
    }
}

#[derive(sqlx::FromRow, Clone)]
pub struct WorkLinkPendingExeRow {
    pub id: i64,
    pub work_id: String,
    pub exe_path: String,
}

impl From<crate::sqliterepository::models::works::WorkLinkPendingExeRow>
    for domain::work_link_pending_exe::WorkLinkPendingExe
{
    fn from(v: crate::sqliterepository::models::works::WorkLinkPendingExeRow) -> Self {
        domain::work_link_pending_exe::WorkLinkPendingExe {
            id: domain::Id::new(v.id as i32),
            work_id: domain::StrId::new(v.work_id),
            exe_path: v.exe_path,
        }
    }
}
