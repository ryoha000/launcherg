use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::domain::{
    collection::{NewCollection, NewCollectionElementDetail},
    Id,
};

#[derive(new)]
pub struct CreateCollection {
    pub name: String,
}

impl TryFrom<CreateCollection> for NewCollection {
    type Error = anyhow::Error;

    fn try_from(c: CreateCollection) -> anyhow::Result<Self> {
        Ok(NewCollection::new(c.name))
    }
}

#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollectionElementDetail {
    pub collection_element_id: i32,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

impl From<CreateCollectionElementDetail> for NewCollectionElementDetail {
    fn from(c: CreateCollectionElementDetail) -> Self {
        NewCollectionElementDetail::new(
            Id::new(c.collection_element_id),
            c.gamename_ruby,
            c.brandname,
            c.brandname_ruby,
            c.sellday,
            c.is_nukige,
        )
    }
}
