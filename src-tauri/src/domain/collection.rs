use chrono::{DateTime, Local};
use derive_new::new;
use serde::{Deserialize, Serialize};

use super::Id;

#[derive(new, Debug)]
pub struct NewCollection {
    pub name: String,
}

// リファクタリング後のCollectionElement
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElement {
    pub id: Id<CollectionElement>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
    // 関連データ
    pub info: Option<CollectionElementInfo>,
    pub paths: Option<CollectionElementPaths>,
    pub install: Option<CollectionElementInstall>,
    pub play: Option<CollectionElementPlay>,
    pub like: Option<CollectionElementLike>,
    pub thumbnail: Option<CollectionElementThumbnail>,
}

// スクレイピング情報（erogamescape由来）
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElementInfo {
    pub id: Id<CollectionElementInfo>,
    pub collection_element_id: Id<CollectionElement>,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// パス情報
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElementPaths {
    pub id: Id<CollectionElementPaths>,
    pub collection_element_id: Id<CollectionElement>,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// インストール情報
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElementInstall {
    pub id: Id<CollectionElementInstall>,
    pub collection_element_id: Id<CollectionElement>,
    pub install_at: DateTime<Local>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// プレイ情報
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElementPlay {
    pub id: Id<CollectionElementPlay>,
    pub collection_element_id: Id<CollectionElement>,
    pub last_play_at: DateTime<Local>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// いいね情報
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElementLike {
    pub id: Id<CollectionElementLike>,
    pub collection_element_id: Id<CollectionElement>,
    pub like_at: DateTime<Local>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// サムネイル情報
#[derive(new, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionElementThumbnail {
    pub id: Id<CollectionElementThumbnail>,
    pub collection_element_id: Id<CollectionElement>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

// 新規作成用の構造体も更新
#[derive(new, Debug)]
pub struct NewCollectionElement {
    pub id: Id<CollectionElement>,
}

#[derive(new, Debug)]
pub struct NewCollectionElementInfo {
    pub collection_element_id: Id<CollectionElement>,
    pub gamename: String,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}

#[derive(new, Debug)]
pub struct NewCollectionElementPaths {
    pub collection_element_id: Id<CollectionElement>,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
}

#[derive(new, Debug)]
pub struct NewCollectionElementInstall {
    pub collection_element_id: Id<CollectionElement>,
    pub install_at: DateTime<Local>,
}

#[derive(new, Debug)]
pub struct NewCollectionElementPlay {
    pub collection_element_id: Id<CollectionElement>,
    pub last_play_at: DateTime<Local>,
}

#[derive(new, Debug)]
pub struct NewCollectionElementLike {
    pub collection_element_id: Id<CollectionElement>,
    pub like_at: DateTime<Local>,
}

#[derive(new, Debug)]
pub struct NewCollectionElementThumbnail {
    pub collection_element_id: Id<CollectionElement>,
    pub thumbnail_width: Option<i32>,
    pub thumbnail_height: Option<i32>,
}

// 後方互換性のために残す（将来的に削除予定）
#[derive(new, Debug, Clone, Serialize, Deserialize)]
pub struct NewCollectionElementDetail {
    pub collection_element_id: Id<CollectionElement>,
    pub gamename_ruby: String,
    pub brandname: String,
    pub brandname_ruby: String,
    pub sellday: String,
    pub is_nukige: bool,
}
