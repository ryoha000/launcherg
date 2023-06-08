use chrono::Local;
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
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: i32,
    pub path: String,
    pub install_at: Option<NaiveDateTime>,
    pub last_play_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl TryFrom<CollectionElementTable> for CollectionElement {
    type Error = anyhow::Error;
    fn try_from(st: CollectionElementTable) -> Result<Self, Self::Error> {
        Ok(CollectionElement::new(
            Id::new(st.id),
            st.gamename,
            st.gamename_ruby,
            st.brandname,
            st.brandname_ruby,
            st.sellday,
            st.is_nukige != 0,
            st.path,
            st.install_at
                .and_then(|v| Some(v.and_utc().with_timezone(&Local))),
            st.last_play_at
                .and_then(|v| Some(v.and_utc().with_timezone(&Local))),
            st.created_at.and_utc().with_timezone(&Local),
            st.updated_at.and_utc().with_timezone(&Local),
        ))
    }
}
