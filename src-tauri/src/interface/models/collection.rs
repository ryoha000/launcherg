use std::sync::Arc;

use derive_new::new;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::domain::{
    self,
    file::{get_icon_path, get_thumbnail_path},
};

#[derive(new, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElement {
    pub id: i32,
    pub erogamescape_id: Option<i32>,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub thumbnail: String,
    pub icon: String,
    pub install_at: Option<String>,
    pub last_play_at: Option<String>,
    pub like_at: Option<String>,
    pub registered_at: String,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub dmm: Option<DmmInfo>,
    pub dlsite: Option<DlsiteInfo>,
    pub install_status: String,
    pub can_play: bool,
    pub can_install: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DmmInfo {
    pub id: i32,
    pub collection_element_id: i32,
    pub category: String,
    pub subcategory: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DlsiteInfo {
    pub id: i32,
    pub collection_element_id: i32,
    pub category: String,
}

impl CollectionElement {
    pub fn from_domain(handle: &Arc<AppHandle>, st: domain::collection::CollectionElement) -> Self {
        // 新しい構造から情報を取得
        let (gamename, gamename_ruby, brandname, brandname_ruby, sellday, is_nukige) =
            if let Some(info) = &st.info {
                (
                    info.gamename.clone(),
                    info.gamename_ruby.clone(),
                    info.brandname.clone(),
                    info.brandname_ruby.clone(),
                    info.sellday.clone(),
                    info.is_nukige,
                )
            } else {
                (
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    "".to_string(),
                    false,
                )
            };

        let (exe_path, lnk_path) = if let Some(paths) = &st.paths {
            (paths.exe_path.clone(), paths.lnk_path.clone())
        } else {
            (None, None)
        };

        let install_at = st.install.as_ref().map(|i| i.install_at.to_rfc3339());
        let last_play_at = st.play.as_ref().map(|p| p.last_play_at.to_rfc3339());
        let like_at = st.like.as_ref().map(|l| l.like_at.to_rfc3339());

        let (thumbnail_width, thumbnail_height) = if let Some(thumbnail) = &st.thumbnail {
            (thumbnail.thumbnail_width, thumbnail.thumbnail_height)
        } else {
            (None, None)
        };

        let install_status = match st.install_status() {
            crate::domain::collection::GameInstallStatus::Installed => "installed",
            crate::domain::collection::GameInstallStatus::OwnedNotInstalled => "owned-not-installed",
            crate::domain::collection::GameInstallStatus::NotOwned => "not-owned",
        };
        let can_play = st.can_play();
        let can_install = st.can_install();

        // DLStore は廃止

        let dmm = st.dmm.as_ref().map(|d| DmmInfo {
            id: d.id.value,
            collection_element_id: d.collection_element_id.value,
            category: d.category.clone(),
            subcategory: d.subcategory.clone(),
        });
        let dlsite = st.dlsite.as_ref().map(|d| DlsiteInfo {
            id: d.id.value,
            collection_element_id: d.collection_element_id.value,
            category: d.category.clone(),
        });

        let erogamescape_id = st.erogamescape.as_ref().map(|m| m.erogamescape_id);
        CollectionElement::new(
            st.id.value,
            erogamescape_id,
            gamename,
            gamename_ruby,
            brandname,
            brandname_ruby,
            sellday,
            is_nukige,
            exe_path,
            lnk_path,
            get_thumbnail_path(handle, &st.id),
            get_icon_path(handle, &st.id),
            install_at,
            last_play_at,
            like_at,
            st.updated_at.to_rfc3339(),
            thumbnail_width,
            thumbnail_height,
            dmm,
            dlsite,
            install_status.to_string(),
            can_play,
            can_install,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct CalculateDistanceKV {
    pub key: String,
    pub value: String,
}

// Serialize/Deserializeが必要なinterface層用の構造体
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableCollectionElement {
    pub id: i32,
    pub created_at: String,
    pub updated_at: String,
    pub info: Option<SerializableCollectionElementInfo>,
    pub paths: Option<SerializableCollectionElementPaths>,
    pub install: Option<SerializableCollectionElementInstall>,
    pub play: Option<SerializableCollectionElementPlay>,
    pub like: Option<SerializableCollectionElementLike>,
    pub thumbnail: Option<SerializableCollectionElementThumbnail>,
    // pub dl_store: Option<SerializableCollectionElementDLStore>,
    pub dmm: Option<SerializableCollectionElementDmm>,
    pub dlsite: Option<SerializableCollectionElementDlsite>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableCollectionElementInfo {
    pub id: i32,
    pub collection_element_id: i32,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableCollectionElementPaths {
    pub id: i32,
    pub collection_element_id: i32,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableCollectionElementInstall {
    pub id: i32,
    pub collection_element_id: i32,
    pub install_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableCollectionElementPlay {
    pub id: i32,
    pub collection_element_id: i32,
    pub last_play_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableCollectionElementLike {
    pub id: i32,
    pub collection_element_id: i32,
    pub like_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableCollectionElementThumbnail {
    pub id: i32,
    pub collection_element_id: i32,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

// DLStore は廃止

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableCollectionElementDmm {
    pub id: i32,
    pub collection_element_id: i32,
    pub category: String,
    pub subcategory: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerializableCollectionElementDlsite {
    pub id: i32,
    pub collection_element_id: i32,
    pub category: String,
}

// domainからinterfaceへの変換メソッド
impl SerializableCollectionElement {
    pub fn from_domain(element: &domain::collection::CollectionElement) -> Self {
        Self {
            id: element.id.value,
            created_at: element.created_at.to_rfc3339(),
            updated_at: element.updated_at.to_rfc3339(),
            info: element.info.as_ref().map(SerializableCollectionElementInfo::from_domain),
            paths: element.paths.as_ref().map(SerializableCollectionElementPaths::from_domain),
            install: element.install.as_ref().map(SerializableCollectionElementInstall::from_domain),
            play: element.play.as_ref().map(SerializableCollectionElementPlay::from_domain),
            like: element.like.as_ref().map(SerializableCollectionElementLike::from_domain),
            thumbnail: element.thumbnail.as_ref().map(SerializableCollectionElementThumbnail::from_domain),
            // dl_store: element.dl_store.as_ref().map(SerializableCollectionElementDLStore::from_domain),
            dmm: element.dmm.as_ref().map(SerializableCollectionElementDmm::from_domain),
            dlsite: element.dlsite.as_ref().map(SerializableCollectionElementDlsite::from_domain),
        }
    }
}

impl SerializableCollectionElementInfo {
    pub fn from_domain(info: &domain::collection::CollectionElementInfo) -> Self {
        Self {
            id: info.id.value,
            collection_element_id: info.collection_element_id.value,
            gamename: info.gamename.clone(),
            gamename_ruby: info.gamename_ruby.clone(),
            brandname: info.brandname.clone(),
            brandname_ruby: info.brandname_ruby.clone(),
            sellday: info.sellday.clone(),
            is_nukige: info.is_nukige,
            created_at: info.created_at.to_rfc3339(),
            updated_at: info.updated_at.to_rfc3339(),
        }
    }
}

impl SerializableCollectionElementPaths {
    pub fn from_domain(paths: &domain::collection::CollectionElementPaths) -> Self {
        Self {
            id: paths.id.value,
            collection_element_id: paths.collection_element_id.value,
            exe_path: paths.exe_path.clone(),
            lnk_path: paths.lnk_path.clone(),
            created_at: paths.created_at.to_rfc3339(),
            updated_at: paths.updated_at.to_rfc3339(),
        }
    }
}

impl SerializableCollectionElementInstall {
    pub fn from_domain(install: &domain::collection::CollectionElementInstall) -> Self {
        Self {
            id: install.id.value,
            collection_element_id: install.collection_element_id.value,
            install_at: install.install_at.to_rfc3339(),
            created_at: install.created_at.to_rfc3339(),
            updated_at: install.updated_at.to_rfc3339(),
        }
    }
}

impl SerializableCollectionElementPlay {
    pub fn from_domain(play: &domain::collection::CollectionElementPlay) -> Self {
        Self {
            id: play.id.value,
            collection_element_id: play.collection_element_id.value,
            last_play_at: play.last_play_at.to_rfc3339(),
            created_at: play.created_at.to_rfc3339(),
            updated_at: play.updated_at.to_rfc3339(),
        }
    }
}

impl SerializableCollectionElementLike {
    pub fn from_domain(like: &domain::collection::CollectionElementLike) -> Self {
        Self {
            id: like.id.value,
            collection_element_id: like.collection_element_id.value,
            like_at: like.like_at.to_rfc3339(),
            created_at: like.created_at.to_rfc3339(),
            updated_at: like.updated_at.to_rfc3339(),
        }
    }
}

impl SerializableCollectionElementThumbnail {
    pub fn from_domain(thumbnail: &domain::collection::CollectionElementThumbnail) -> Self {
        Self {
            id: thumbnail.id.value,
            collection_element_id: thumbnail.collection_element_id.value,
            thumbnail_width: thumbnail.thumbnail_width,
            thumbnail_height: thumbnail.thumbnail_height,
            created_at: thumbnail.created_at.to_rfc3339(),
            updated_at: thumbnail.updated_at.to_rfc3339(),
        }
    }
}

impl SerializableCollectionElementDmm {
    pub fn from_domain(dmm: &domain::collection::CollectionElementDMM) -> Self {
        Self {
            id: dmm.id.value,
            collection_element_id: dmm.collection_element_id.value,
            category: dmm.category.clone(),
            subcategory: dmm.subcategory.clone(),
        }
    }
}

impl SerializableCollectionElementDlsite {
    pub fn from_domain(dlsite: &domain::collection::CollectionElementDLsite) -> Self {
        Self {
            id: dlsite.id.value,
            collection_element_id: dlsite.collection_element_id.value,
            category: dlsite.category.clone(),
        }
    }
}
