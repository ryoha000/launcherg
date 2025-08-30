use serde::{Deserialize, Serialize};

use crate::Id;

use crate::collection::CollectionElement;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Work {
    pub id: Id<Work>,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DmmWork {
    pub id: Id<DmmWork>,
    pub title: String,
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DlsiteWork {
    pub id: Id<DlsiteWork>,
    pub title: String,
    pub store_id: String,
    pub category: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDmmWork {
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
    pub work_id: Id<Work>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewDlsiteWork {
    pub store_id: String,
    pub category: String,
    pub work_id: Id<Work>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewWork {
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkDetails {
    pub work: Work,
    pub dmm: Option<DmmWork>,
    pub dlsite: Option<DlsiteWork>,
    pub collection_element_id: Option<Id<CollectionElement>>,
    pub is_dmm_omitted: bool,
    pub is_dlsite_omitted: bool,
    pub is_dmm_pack: bool,
}


