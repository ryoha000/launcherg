use chrono::{DateTime, Local};
use derive_new::new;

use crate::Id;

#[derive(new, Clone, Debug, PartialEq)]
pub struct ErogamescapeInformation {
    pub id: Id<ErogamescapeInformation>,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(new, Clone, Debug, PartialEq)]
pub struct NewErogamescapeInformation {
    pub erogamescape_id: i32,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}
