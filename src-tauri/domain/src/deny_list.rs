use serde::{Deserialize, Serialize};
use crate::Id;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum StoreType {
    Unspecified = 0,
    Dmm = 1,
    Dlsite = 2,
}

impl TryFrom<i32> for StoreType {
    type Error = &'static str;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(StoreType::Dmm),
            2 => Ok(StoreType::Dlsite),
            0 => Ok(StoreType::Unspecified),
            _ => Err("invalid store type"),
        }
    }
}

impl From<StoreType> for i32 {
    fn from(value: StoreType) -> Self { value as i32 }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DenyListEntry {
    pub id: Id<DenyListEntry>,
    pub store_type: StoreType,
    pub store_id: String,
    pub name: String,
}


