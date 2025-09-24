use chrono::Local;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

use domain::{
    collection::{
        CollectionElement, CollectionElementErogamescape, CollectionElementInstall,
        CollectionElementLike, CollectionElementPaths, CollectionElementPlay,
        CollectionElementThumbnail,
    },
    Id,
};

#[derive(FromRow, Clone)]
pub struct CollectionElementDetailsRow {
    pub ce_id: i32,
    pub ce_gamename: String,
    pub ce_created_at: NaiveDateTime,
    pub ce_updated_at: NaiveDateTime,

    pub info_id: Option<i32>,
    pub info_gamename_ruby: Option<String>,
    pub info_sellday: Option<String>,
    pub info_is_nukige: Option<i32>,
    pub info_brandname: Option<String>,
    pub info_brandname_ruby: Option<String>,
    pub info_created_at: Option<NaiveDateTime>,
    pub info_updated_at: Option<NaiveDateTime>,

    pub paths_id: Option<i32>,
    pub paths_exe_path: Option<String>,
    pub paths_lnk_path: Option<String>,
    pub paths_created_at: Option<NaiveDateTime>,
    pub paths_updated_at: Option<NaiveDateTime>,

    pub install_id: Option<i32>,
    pub install_install_at: Option<NaiveDateTime>,
    pub install_created_at: Option<NaiveDateTime>,
    pub install_updated_at: Option<NaiveDateTime>,

    pub play_id: Option<i32>,
    pub play_last_play_at: Option<NaiveDateTime>,
    pub play_created_at: Option<NaiveDateTime>,
    pub play_updated_at: Option<NaiveDateTime>,

    pub like_id: Option<i32>,
    pub like_like_at: Option<NaiveDateTime>,
    pub like_created_at: Option<NaiveDateTime>,
    pub like_updated_at: Option<NaiveDateTime>,

    pub thumbnail_id: Option<i32>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub thumbnail_created_at: Option<NaiveDateTime>,
    pub thumbnail_updated_at: Option<NaiveDateTime>,

    pub egs_id: Option<i32>,
    pub egs_erogamescape_id: Option<i32>,
    pub egs_created_at: Option<NaiveDateTime>,
    pub egs_updated_at: Option<NaiveDateTime>,
}

#[derive(FromRow)]
pub struct CollectionElementTable {
    pub id: i32,
    pub gamename: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(FromRow)]
pub struct CollectionElementInfoTable {
    pub id: i32,
    pub collection_element_id: i32,
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
            st.gamename,
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
            None, // info は別途取得
            None, // paths は別途取得
            None, // install は別途取得
            None, // play は別途取得
            None, // like は別途取得
            None, // thumbnail は別途取得
            None, // dmm は別途取得
            None, // dlsite は別途取得
            None, // erogamescape は別途取得
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

// DLStoreは廃止のため、ドメイン変換は提供しない

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

impl From<CollectionElementDetailsRow> for CollectionElement {
    fn from(r: CollectionElementDetailsRow) -> Self {
        let id = Id::new(r.ce_id);
        let mut element = CollectionElement::new(
            id.clone(),
            r.ce_gamename,
            r.ce_created_at.and_utc().with_timezone(&Local),
            r.ce_updated_at.and_utc().with_timezone(&Local),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );


        if let Some(paths_id) = r.paths_id {
            if let (Some(created), Some(updated)) = (r.paths_created_at, r.paths_updated_at) {
                element.paths = Some(CollectionElementPaths::new(
                    Id::new(paths_id),
                    id.clone(),
                    r.paths_exe_path,
                    r.paths_lnk_path,
                    created.and_utc().with_timezone(&Local),
                    updated.and_utc().with_timezone(&Local),
                ));
            }
        }

        if let Some(install_id) = r.install_id {
            if let (Some(install_at), Some(created), Some(updated)) = (
                r.install_install_at,
                r.install_created_at,
                r.install_updated_at,
            ) {
                element.install = Some(CollectionElementInstall::new(
                    Id::new(install_id),
                    id.clone(),
                    install_at.and_utc().with_timezone(&Local),
                    created.and_utc().with_timezone(&Local),
                    updated.and_utc().with_timezone(&Local),
                ));
            }
        }

        if let Some(play_id) = r.play_id {
            if let (Some(last_play_at), Some(created), Some(updated)) =
                (r.play_last_play_at, r.play_created_at, r.play_updated_at)
            {
                element.play = Some(CollectionElementPlay::new(
                    Id::new(play_id),
                    id.clone(),
                    last_play_at.and_utc().with_timezone(&Local),
                    created.and_utc().with_timezone(&Local),
                    updated.and_utc().with_timezone(&Local),
                ));
            }
        }

        if let Some(like_id) = r.like_id {
            if let (Some(like_at), Some(created), Some(updated)) =
                (r.like_like_at, r.like_created_at, r.like_updated_at)
            {
                element.like = Some(CollectionElementLike::new(
                    Id::new(like_id),
                    id.clone(),
                    like_at.and_utc().with_timezone(&Local),
                    created.and_utc().with_timezone(&Local),
                    updated.and_utc().with_timezone(&Local),
                ));
            }
        }

        if let Some(thumbnail_id) = r.thumbnail_id {
            if let (Some(created), Some(updated)) = (r.thumbnail_created_at, r.thumbnail_updated_at)
            {
                element.thumbnail = Some(CollectionElementThumbnail::new(
                    Id::new(thumbnail_id),
                    id.clone(),
                    r.thumbnail_width,
                    r.thumbnail_height,
                    created.and_utc().with_timezone(&Local),
                    updated.and_utc().with_timezone(&Local),
                ));
            }
        }

        if let Some(egs_row_id) = r.egs_id {
            if let (Some(egs_id), Some(created), Some(updated)) =
                (r.egs_erogamescape_id, r.egs_created_at, r.egs_updated_at)
            {
                element.erogamescape = Some(CollectionElementErogamescape::new(
                    Id::new(egs_row_id),
                    id.clone(),
                    egs_id,
                    created.and_utc().with_timezone(&Local),
                    updated.and_utc().with_timezone(&Local),
                ));
            }
        }

        element
    }
}
