use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

use crate::domain::{
    collection::{Collection, CollectionElement},
    Id,
};

#[derive(FromRow)]
pub struct CollectionTable {
    pub id: i32,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl TryFrom<CollectionTable> for Collection {
    type Error = anyhow::Error;
    fn try_from(st: CollectionTable) -> Result<Self, Self::Error> {
        Ok(Collection::new(
            Id::new(st.id),
            st.name,
            st.created_at,
            st.updated_at,
        ))
    }
}

#[derive(FromRow)]
pub struct CollectionElementTable {
    pub id: i32,
    pub gamename: String,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl TryFrom<CollectionElementTable> for CollectionElement {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementTable) -> Result<Self, Self::Error> {
        Ok(CollectionElement::new(
            Id::new(st.id),
            st.gamename,
            st.path,
            st.created_at,
            st.updated_at,
        ))
    }
}
