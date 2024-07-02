use derive_new::new;
use serde::Serialize;

use crate::domain::collection::CollectionElement;

#[derive(new, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub erogame_scape_id: i32,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub install_at: Option<String>,
    pub last_play_at: Option<String>,
    pub like_at: Option<String>,
    pub updated_at: String,
}

impl From<CollectionElement> for Game {
    fn from(st: CollectionElement) -> Self {
        Game::new(
            st.id.value,
            st.gamename,
            st.gamename_ruby,
            st.brandname,
            st.brandname_ruby,
            st.sellday,
            st.is_nukige,
            st.exe_path,
            st.lnk_path,
            st.install_at.and_then(|v| Some(v.to_rfc3339())),
            st.last_play_at.and_then(|v| Some(v.to_rfc3339())),
            st.like_at.and_then(|v| Some(v.to_rfc3339())),
            st.updated_at.to_rfc3339(),
        )
    }
}
