use derive_new::new;
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::NaiveDateTime;

pub type CollectionID = i32;
#[derive(new, Debug, Serialize, Deserialize, Clone)]
pub struct Collection {
    pub id: CollectionID,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(new, Debug)]
pub struct NewCollection {
    pub name: String,
}

pub type UpdateNameCollection = NewCollection;

pub type CollectionElementID = i32;
#[derive(new, Debug, Serialize, Deserialize)]
pub struct CollectionElement {
    pub id: CollectionElementID,
    pub gamename: String,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
