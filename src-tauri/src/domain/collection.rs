use chrono::{DateTime, Local};
use derive_new::new;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDateTime;

use super::Id;

#[derive(new, Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: Id<Collection>,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(new, Debug)]
pub struct NewCollection {
    pub name: String,
}

#[derive(new, Debug)]
pub struct UpdateCollection {
    pub id: Id<Collection>,
    pub name: String,
}

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElement {
    pub id: Id<CollectionElement>,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
    pub path: String,
    pub install_at: Option<DateTime<Local>>,
    pub last_play_at: Option<DateTime<Local>>,
    pub like_at: Option<DateTime<Local>>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(new, Debug)]
pub struct NewCollectionElement {
    pub id: Id<CollectionElement>,
    pub gamename: String,
    pub path: String,
    pub install_at: Option<DateTime<Local>>,
}

#[derive(new, Debug, Clone, Serialize, Deserialize)]
pub struct NewCollectionElementDetail {
    pub collection_element_id: Id<CollectionElement>,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}
