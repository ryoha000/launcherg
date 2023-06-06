use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::domain;

#[derive(derive_new::new, Debug, Serialize, Deserialize, Clone)]
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

#[derive(Serialize)]
pub struct CollectionElement {
    pub id: i32,
    pub gamename: String,
    pub path: String,
    pub icon: String,
}

#[derive(Serialize, Deserialize)]
pub struct CalculateDistanceKV {
    pub key: String,
    pub value: String,
}
