use derive_new::new;
use serde::{Deserialize, Serialize};

use crate::domain::{self, all_game_cache::AllGameCacheOneWithThumbnailUrl};

#[derive(new, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllGameCacheOne {
    pub id: i32,
    pub gamename: String,
    pub thumbnail_url: String,
}

impl From<AllGameCacheOneWithThumbnailUrl> for AllGameCacheOne {
    fn from(st: AllGameCacheOneWithThumbnailUrl) -> Self {
        AllGameCacheOne::new(st.id, st.gamename, st.thumbnail_url)
    }
}

impl From<AllGameCacheOne> for domain::all_game_cache::AllGameCacheOne {
    fn from(st: AllGameCacheOne) -> Self {
        domain::all_game_cache::AllGameCacheOne::new(st.id, st.gamename)
    }
}

impl From<AllGameCacheOne> for domain::all_game_cache::NewAllGameCacheOne {
    fn from(st: AllGameCacheOne) -> Self {
        domain::all_game_cache::NewAllGameCacheOne::new(st.id, st.gamename, st.thumbnail_url)
    }
}
