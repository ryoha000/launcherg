use serde::{Deserialize, Serialize};

pub struct NetWork {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErogamescapeIDNamePair {
    pub id: i32,
    pub gamename: String,
}
