use std::sync::Arc;

use derive_new::new;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::domain;
use crate::domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};

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
    pub fn from_domain(_handle: &Arc<AppHandle>, st: domain::collection::CollectionElement) -> Self {
        let (gamename_ruby, brandname, brandname_ruby, sellday, is_nukige) =
            if let Some(info) = &st.info {
                (
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
        let resolver = DirsSavePathResolver::default();
        CollectionElement::new(
            st.id.value,
            erogamescape_id,
            st.gamename,
            gamename_ruby,
            brandname,
            brandname_ruby,
            sellday,
            is_nukige,
            exe_path,
            lnk_path,
            resolver.thumbnail_png_path(st.id.value),
            resolver.icon_png_path(st.id.value),
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
