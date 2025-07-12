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
}

impl CollectionElement {
    pub fn from_domain(handle: &Arc<AppHandle>, st: domain::collection::CollectionElement) -> Self {
        CollectionElement::new(
            st.id.value,
            st.gamename,
            st.gamename_ruby,
            st.brandname,
            st.brandname_ruby,
            st.sellday,
            st.is_nukige,
            st.exe_path,
            st.lnk_path,
            get_thumbnail_path(handle, &st.id),
            get_icon_path(handle, &st.id),
            st.install_at.and_then(|v| Some(v.to_rfc3339())),
            st.last_play_at.and_then(|v| Some(v.to_rfc3339())),
            st.like_at.and_then(|v| Some(v.to_rfc3339())),
            st.updated_at.to_rfc3339(),
            st.thumbnail_width,
            st.thumbnail_height,
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct CalculateDistanceKV {
    pub key: String,
    pub value: String,
}

// the payload type must implement `Serialize` and `Clone`.
#[derive(new, Clone, serde::Serialize)]
pub struct ProgressPayload {
    pub message: String,
}

#[derive(new, Clone, serde::Serialize)]
pub struct ProgressLivePayload {
    pub max: Option<i32>,
}
