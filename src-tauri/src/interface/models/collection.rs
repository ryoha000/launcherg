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
    pub dl_store: Option<DLStoreInfo>,
    pub install_status: String,
    pub can_play: bool,
    pub can_install: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DLStoreInfo {
    pub id: i32,
    pub collection_element_id: i32,
    pub store_id: String,
    pub store_type: String,
    pub store_name: String,
    pub purchase_url: String,
    pub is_owned: bool,
    pub purchase_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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

        let dl_store = st.dl_store.map(|dl_store| DLStoreInfo {
            id: dl_store.id.value,
            collection_element_id: dl_store.collection_element_id.value,
            store_id: dl_store.store_id,
            store_type: match dl_store.store_type {
                crate::domain::collection::DLStoreType::DMM => "DMM".to_string(),
                crate::domain::collection::DLStoreType::DLSite => "DLSite".to_string(),
            },
            store_name: dl_store.store_name,
            purchase_url: dl_store.purchase_url,
            is_owned: dl_store.is_owned,
            purchase_date: dl_store.purchase_date.map(|dt| dt.to_rfc3339()),
            created_at: dl_store.created_at.to_rfc3339(),
            updated_at: dl_store.updated_at.to_rfc3339(),
        });

        CollectionElement::new(
            st.id.value,
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
            dl_store,
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
