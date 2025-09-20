use chrono::{DateTime, Local};
use derive_new::new;
use serde::{Deserialize, Serialize};

use super::Id;

#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElement {
    pub id: Id<CollectionElement>,
    pub gamename: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    // 関連データ
    pub info: Option<CollectionElementInfo>,
    pub paths: Option<CollectionElementPaths>,
    pub install: Option<CollectionElementInstall>,
    pub play: Option<CollectionElementPlay>,
    pub like: Option<CollectionElementLike>,
    pub thumbnail: Option<CollectionElementThumbnail>,
    pub dmm: Option<CollectionElementDMM>,
    pub dlsite: Option<CollectionElementDLsite>,
    pub erogamescape: Option<CollectionElementErogamescape>,
}

// スクレイピング情報（erogamescape由来）
#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElementInfo {
    pub id: Id<CollectionElementInfo>,
    pub collection_element_id: Id<CollectionElement>,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// パス情報
#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElementPaths {
    pub id: Id<CollectionElementPaths>,
    pub collection_element_id: Id<CollectionElement>,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// インストール情報
#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElementInstall {
    pub id: Id<CollectionElementInstall>,
    pub collection_element_id: Id<CollectionElement>,
    pub install_at: DateTime<Local>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// プレイ情報
#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElementPlay {
    pub id: Id<CollectionElementPlay>,
    pub collection_element_id: Id<CollectionElement>,
    pub last_play_at: DateTime<Local>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// いいね情報
#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElementLike {
    pub id: Id<CollectionElementLike>,
    pub collection_element_id: Id<CollectionElement>,
    pub like_at: DateTime<Local>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// サムネイル情報
#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElementThumbnail {
    pub id: Id<CollectionElementThumbnail>,
    pub collection_element_id: Id<CollectionElement>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// ErogameScape ID マッピング
#[derive(new, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollectionElementErogamescape {
    pub id: Id<CollectionElementErogamescape>,
    pub collection_element_id: Id<CollectionElement>,
    pub erogamescape_id: i32,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElement {
    pub id: Id<CollectionElement>,
    pub gamename: String,
}

// ファイルスキャン用の関連データ付きコレクション要素
#[derive(new, Clone, Debug)]
pub struct ScannedGameElement {
    pub erogamescape_id: i32,
    pub gamename: String,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub install_at: Option<DateTime<Local>>,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementInfo {
    pub collection_element_id: Id<CollectionElement>,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementPaths {
    pub collection_element_id: Id<CollectionElement>,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementInstall {
    pub collection_element_id: Id<CollectionElement>,
    pub install_at: DateTime<Local>,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementPlay {
    pub collection_element_id: Id<CollectionElement>,
    pub last_play_at: DateTime<Local>,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementLike {
    pub collection_element_id: Id<CollectionElement>,
    pub like_at: DateTime<Local>,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementThumbnail {
    pub collection_element_id: Id<CollectionElement>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
}

// DMM マッピング
#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElementDMM {
    pub id: Id<CollectionElementDMM>,
    pub collection_element_id: Id<CollectionElement>,
    pub category: String,
    pub subcategory: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// DLsite マッピング
#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElementDLsite {
    pub id: Id<CollectionElementDLsite>,
    pub collection_element_id: Id<CollectionElement>,
    pub category: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementDMM {
    pub collection_element_id: Id<CollectionElement>,
    pub category: String,
    pub subcategory: String,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementDLsite {
    pub collection_element_id: Id<CollectionElement>,
    pub category: String,
}
