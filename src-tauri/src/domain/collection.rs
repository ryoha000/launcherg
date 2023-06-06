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
pub struct CollectionElement {
    pub id: Id<CollectionElement>,
    pub gamename: String,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(new, Debug)]
pub struct NewCollectionElement {
    pub id: Id<CollectionElement>,
    pub gamename: String,
    pub path: String,
}
