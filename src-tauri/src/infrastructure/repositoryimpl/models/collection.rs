use chrono::Local;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::domain::{
    collection::{
        CollectionElement, CollectionElementInfo, CollectionElementInstall, CollectionElementLike,
        CollectionElementPaths, CollectionElementPlay, CollectionElementThumbnail,
        CollectionElementDLStore, DLStoreType, CollectionElementErogamescape,
    },
    Id,
};

#[derive(FromRow)]
pub struct CollectionElementTable {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct CollectionElementInfoTable {
    pub id: i32,
    pub collection_element_id: i32,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct CollectionElementPathsTable {
    pub id: i32,
    pub collection_element_id: i32,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct CollectionElementInstallTable {
    pub id: i32,
    pub collection_element_id: i32,
    pub install_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct CollectionElementPlayTable {
    pub id: i32,
    pub collection_element_id: i32,
    pub last_play_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct CollectionElementLikeTable {
    pub id: i32,
    pub collection_element_id: i32,
    pub like_at: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct CollectionElementThumbnailTable {
    pub id: i32,
    pub collection_element_id: i32,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct CollectionElementDLStoreTable {
    pub id: i32,
    pub collection_element_id: i32,
    pub store_id: String,
    pub store_type: String,
    pub store_name: String,
    pub purchase_url: String,
    pub is_owned: i32,
    pub purchase_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct CollectionElementErogamescapeTable {
    pub id: i32,
    pub collection_element_id: i32,
    pub erogamescape_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// ドメインモデルへの変換実装
impl TryFrom<CollectionElementTable> for CollectionElement {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementTable) -> Result<Self, Self::Error> {
        Ok(CollectionElement::new(
            Id::new(st.id),
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
            None, // info は別途取得
            None, // paths は別途取得
            None, // install は別途取得
            None, // play は別途取得
            None, // like は別途取得
            None, // thumbnail は別途取得
            None, // dl_store は別途取得
            None, // erogamescape は別途取得
        ))
    }
}

impl TryFrom<CollectionElementInfoTable> for CollectionElementInfo {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementInfoTable) -> Result<Self, Self::Error> {
        Ok(CollectionElementInfo::new(
            Id::new(st.id),
            Id::new(st.collection_element_id),
            st.gamename,
            st.gamename_ruby,
            st.brandname,
            st.brandname_ruby,
            st.sellday,
            st.is_nukige != 0,
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}

impl TryFrom<CollectionElementPathsTable> for CollectionElementPaths {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementPathsTable) -> Result<Self, Self::Error> {
        Ok(CollectionElementPaths::new(
            Id::new(st.id),
            Id::new(st.collection_element_id),
            st.exe_path,
            st.lnk_path,
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}

impl TryFrom<CollectionElementInstallTable> for CollectionElementInstall {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementInstallTable) -> Result<Self, Self::Error> {
        Ok(CollectionElementInstall::new(
            Id::new(st.id),
            Id::new(st.collection_element_id),
            st.install_at.and_utc().with_timezone(&Local),
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}

impl TryFrom<CollectionElementPlayTable> for CollectionElementPlay {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementPlayTable) -> Result<Self, Self::Error> {
        Ok(CollectionElementPlay::new(
            Id::new(st.id),
            Id::new(st.collection_element_id),
            st.last_play_at.and_utc().with_timezone(&Local),
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}

impl TryFrom<CollectionElementLikeTable> for CollectionElementLike {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementLikeTable) -> Result<Self, Self::Error> {
        Ok(CollectionElementLike::new(
            Id::new(st.id),
            Id::new(st.collection_element_id),
            st.like_at.and_utc().with_timezone(&Local),
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}

impl TryFrom<CollectionElementThumbnailTable> for CollectionElementThumbnail {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementThumbnailTable) -> Result<Self, Self::Error> {
        Ok(CollectionElementThumbnail::new(
            Id::new(st.id),
            Id::new(st.collection_element_id),
            st.thumbnail_width,
            st.thumbnail_height,
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}

impl TryFrom<CollectionElementDLStoreTable> for CollectionElementDLStore {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementDLStoreTable) -> Result<Self, Self::Error> {
        let store_type = match st.store_type.as_str() {
            "DMM" => DLStoreType::DMM,
            "DLSite" => DLStoreType::DLSite,
            _ => return Err(anyhow::anyhow!("Unknown store type: {}", st.store_type)),
        };

        Ok(CollectionElementDLStore::new(
            Id::new(st.id),
            Id::new(st.collection_element_id),
            st.store_id,
            store_type,
            st.store_name,
            st.purchase_url,
            st.is_owned != 0,
            st.purchase_date
                .map(|dt| dt.and_utc().with_timezone(&Local)),
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}

impl TryFrom<CollectionElementErogamescapeTable> for CollectionElementErogamescape {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementErogamescapeTable) -> Result<Self, Self::Error> {
        Ok(CollectionElementErogamescape::new(
            Id::new(st.id),
            Id::new(st.collection_element_id),
            st.erogamescape_id,
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}
