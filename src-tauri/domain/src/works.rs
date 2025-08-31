use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::Id;

use crate::collection::CollectionElement;

#[derive(new, Clone, Debug, Serialize, Deserialize)]
pub struct Work {
    pub id: Id<Work>,
    pub title: String,
}

#[derive(new, Clone, Debug, Serialize, Deserialize)]
pub struct DmmWork {
    pub id: Id<DmmWork>,
    pub work_id: Id<Work>,
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
}

#[derive(new, Clone, Debug, Serialize, Deserialize)]
pub struct DlsiteWork {
    pub id: Id<DlsiteWork>,
    pub work_id: Id<Work>,
    pub store_id: String,
    pub category: String,
}

#[derive(new, Clone, Debug, Serialize, Deserialize)]
pub struct NewDmmWork {
    pub store_id: String,
    pub category: String,
    pub subcategory: String,
    pub work_id: Id<Work>,
}

#[derive(new, Clone, Debug, Serialize, Deserialize)]
pub struct NewDlsiteWork {
    pub store_id: String,
    pub category: String,
    pub work_id: Id<Work>,
}

#[derive(new, Clone, Debug, Serialize, Deserialize)]
pub struct NewWork {
    pub title: String,
}

#[derive(new, Clone, Debug, Serialize, Deserialize)]
pub struct WorkDetails {
    pub work: Work,
    pub dmm: Option<DmmWork>,
    pub dlsite: Option<DlsiteWork>,
    pub collection_element_id: Option<Id<CollectionElement>>,
    pub is_dmm_omitted: bool,
    pub is_dlsite_omitted: bool,
    pub is_dmm_pack: bool,
}


