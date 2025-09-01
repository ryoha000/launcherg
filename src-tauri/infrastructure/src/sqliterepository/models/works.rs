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
}

impl TryFrom<crate::sqliterepository::models::works::DmmWorkTable> for domain::works::DmmWork {
    type Error = anyhow::Error;
    fn try_from(v: crate::sqliterepository::models::works::DmmWorkTable) -> Result<Self, Self::Error> {
        Ok(domain::works::DmmWork {
            id: domain::Id::new(v.id as i32),
            work_id: domain::Id::new(v.work_id as i32),
            store_id: v.store_id,
            category: v.category,
            subcategory: v.subcategory,
        })
    }
}

impl TryFrom<crate::sqliterepository::models::works::DlsiteWorkTable> for domain::works::DlsiteWork {
    type Error = anyhow::Error;
    fn try_from(v: crate::sqliterepository::models::works::DlsiteWorkTable) -> Result<Self, Self::Error> {
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
        Ok(domain::works::Work { id: domain::Id::new(v.id as i32), title: v.title })
    }
}


