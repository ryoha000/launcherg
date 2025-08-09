use crate::domain::{
    collection::{
        CollectionElement, CollectionElementInfo, CollectionElementInstall, CollectionElementLike,
        CollectionElementPaths, CollectionElementPlay, CollectionElementThumbnail,
        CollectionElementDLStore, CollectionElementErogamescape, NewCollectionElement, NewCollectionElementInfo, 
        NewCollectionElementInstall, NewCollectionElementLike, NewCollectionElementPaths, 
        NewCollectionElementPlay, NewCollectionElementThumbnail, NewCollectionElementDLStore,
        DLStoreType,
    },
    Id,
};
use anyhow::Result;
use chrono::{DateTime, Local};

#[cfg_attr(test, mockall::automock)]
pub trait CollectionRepository {
    // CollectionElement基本操作
    async fn get_all_elements(&self) -> Result<Vec<CollectionElement>>;
    async fn get_element_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElement>>;
    async fn upsert_collection_element(&self, new_element: &NewCollectionElement) -> Result<()>;
    async fn delete_collection_element(&self, element_id: &Id<CollectionElement>) -> Result<()>;

    // CollectionElementInfo操作
    async fn upsert_collection_element_info(&self, info: &NewCollectionElementInfo) -> Result<()>;
    async fn get_element_info_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementInfo>>;
    async fn get_not_registered_info_element_ids(&self) -> Result<Vec<Id<CollectionElement>>>;

    // CollectionElementPaths操作
    async fn upsert_collection_element_paths(
        &self,
        paths: &NewCollectionElementPaths,
    ) -> Result<()>;
    async fn get_element_paths_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementPaths>>;

    // CollectionElementInstall操作
    async fn upsert_collection_element_install(
        &self,
        install: &NewCollectionElementInstall,
    ) -> Result<()>;
    async fn get_element_install_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementInstall>>;

    // CollectionElementPlay操作
    async fn upsert_collection_element_play(&self, play: &NewCollectionElementPlay) -> Result<()>;
    async fn get_element_play_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementPlay>>;
    async fn update_element_last_play_at_by_id(
        &self,
        id: &Id<CollectionElement>,
        last_play_at: DateTime<Local>,
    ) -> Result<()>;

    // CollectionElementLike操作
    async fn upsert_collection_element_like(&self, like: &NewCollectionElementLike) -> Result<()>;
    async fn delete_collection_element_like_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<()>;
    async fn get_element_like_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementLike>>;
    async fn update_element_like_at_by_id(
        &self,
        id: &Id<CollectionElement>,
        like_at: Option<DateTime<Local>>,
    ) -> Result<()>;

    // CollectionElementThumbnail操作
    async fn upsert_collection_element_thumbnail(
        &self,
        thumbnail: &NewCollectionElementThumbnail,
    ) -> Result<()>;
    async fn get_element_thumbnail_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementThumbnail>>;
    async fn upsert_collection_element_thumbnail_size(
        &self,
        id: &Id<CollectionElement>,
        width: i32,
        height: i32,
    ) -> Result<()>;
    async fn get_null_thumbnail_size_element_ids(&self) -> Result<Vec<Id<CollectionElement>>>;

    // CollectionElementDLStore操作
    async fn upsert_collection_element_dl_store(
        &self,
        dl_store: &NewCollectionElementDLStore,
    ) -> Result<()>;
    async fn get_element_dl_store_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementDLStore>>;
    async fn get_element_erogamescape_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementErogamescape>>;
    async fn get_element_dl_store_by_store_id(
        &self,
        store_id: &str,
        store_type: &DLStoreType,
    ) -> Result<Option<CollectionElementDLStore>>;
    async fn update_collection_element_dl_store(
        &self,
        dl_store: &CollectionElementDLStore,
    ) -> Result<()>;
    async fn delete_collection_element_dl_store(
        &self,
        id: &Id<CollectionElementDLStore>,
    ) -> Result<()>;
    async fn get_uninstalled_owned_games(&self) -> Result<Vec<CollectionElement>>;

    // EGS ID マッピング操作
    async fn get_collection_id_by_erogamescape_id(
        &self,
        erogamescape_id: i32,
    ) -> Result<Option<Id<CollectionElement>>>;
    async fn upsert_erogamescape_map(
        &self,
        collection_element_id: &Id<CollectionElement>,
        erogamescape_id: i32,
    ) -> Result<()>;
    // 正のIDを自前採番して即座に collection_elements に予約挿入する
    async fn allocate_new_collection_element_id(&self) -> Result<Id<CollectionElement>>;

    // 逆引き: collection_element_id -> erogamescape_id
    async fn get_erogamescape_id_by_collection_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<i32>>;
}
