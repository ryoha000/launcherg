use chrono::{DateTime, Local};
use derive_new::new;

use super::Id;

#[derive(new, Clone, Debug, PartialEq)]
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
    pub dl_store: Option<CollectionElementDLStore>,
}

// スクレイピング情報（erogamescape由来）
#[derive(new, Clone, Debug, PartialEq)]
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

#[derive(new, Clone, Debug)]
pub struct NewCollectionElement {
    pub id: Id<CollectionElement>,
}

// ファイルスキャン用の関連データ付きコレクション要素
#[derive(new, Clone, Debug)]
pub struct ScannedGameElement {
    pub id: Id<CollectionElement>,
    pub exe_path: Option<String>,
    pub lnk_path: Option<String>,
    pub install_at: Option<DateTime<Local>>,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementInfo {
    pub collection_element_id: Id<CollectionElement>,
    pub gamename: String,
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

// DL版ストア情報
#[derive(new, Clone, Debug, PartialEq)]
pub struct CollectionElementDLStore {
    pub id: Id<CollectionElementDLStore>,
    pub collection_element_id: Id<CollectionElement>,
    pub store_id: String,
    pub store_type: DLStoreType,
    pub store_name: String,
    pub purchase_url: String,
    pub is_owned: bool,
    pub purchase_date: Option<DateTime<Local>>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DLStoreType {
    DMM,
    DLSite,
}

#[derive(new, Clone, Debug)]
pub struct NewCollectionElementDLStore {
    pub collection_element_id: Id<CollectionElement>,
    pub store_id: String,
    pub store_type: DLStoreType,
    pub store_name: String,
    pub purchase_url: String,
    pub is_owned: bool,
    pub purchase_date: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameInstallStatus {
    Installed,
    OwnedNotInstalled,
    NotOwned,
}

impl CollectionElement {
    pub fn install_status(&self) -> GameInstallStatus {
        match (&self.paths, &self.dl_store) {
            (Some(paths), _) if paths.exe_path.is_some() || paths.lnk_path.is_some() => {
                GameInstallStatus::Installed
            }
            (None, Some(dl_store)) if dl_store.is_owned => {
                GameInstallStatus::OwnedNotInstalled
            }
            _ => GameInstallStatus::NotOwned,
        }
    }

    pub fn can_play(&self) -> bool {
        matches!(self.install_status(), GameInstallStatus::Installed)
    }

    pub fn can_install(&self) -> bool {
        matches!(self.install_status(), GameInstallStatus::OwnedNotInstalled)
    }
}
