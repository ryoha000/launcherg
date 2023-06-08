use chrono::NaiveDateTime;
use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::domain::{self, file::get_icon_path};

#[derive(new, Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<domain::collection::Collection> for Collection {
    fn from(st: domain::collection::Collection) -> Self {
        Collection::new(st.id.value, st.name, st.created_at, st.updated_at)
    }
}

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
    pub path: String,
    pub icon: String,
    pub install_at: Option<String>,
    pub last_play_at: Option<String>,
}

impl From<domain::collection::CollectionElement> for CollectionElement {
    fn from(st: domain::collection::CollectionElement) -> Self {
        CollectionElement::new(
            st.id.value,
            st.gamename,
            st.gamename_ruby,
            st.brandname,
            st.brandname_ruby,
            st.sellday,
            st.is_nukige,
            st.path,
            get_icon_path(&st.id),
            st.install_at.and_then(|v| Some(v.to_string())),
            st.last_play_at.and_then(|v| Some(v.to_string())),
        )
    }
}

#[derive(Serialize, Deserialize)]
pub struct CalculateDistanceKV {
    pub key: String,
    pub value: String,
}
