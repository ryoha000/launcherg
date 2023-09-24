use crate::domain::all_game_cache::AllGameCacheOne;

pub struct Metadata {
    pub exe_path: String,
    pub icon_path: String,
}

pub struct NewElementContext {
    pub metadata: Metadata,
    pub game_cache: AllGameCacheOne,
    pub lnk_path: Option<String>,
    pub exe_path: Option<String>,
}
