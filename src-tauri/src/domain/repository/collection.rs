use crate::domain::{
    collection::{
        CollectionElement, CollectionElementInfo, CollectionElementInstall, CollectionElementLike,
        CollectionElementPaths, CollectionElementPlay, CollectionElementThumbnail,
        NewCollectionElement, NewCollectionElementInfo, NewCollectionElementInstall,
        NewCollectionElementLike, NewCollectionElementPaths, NewCollectionElementPlay,
        NewCollectionElementThumbnail,
    },
    Id,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};

#[async_trait]
pub trait CollectionRepository {
    // CollectionElement基本操作
    async fn get_all_elements(&self) -> Result<Vec<CollectionElement>>;
    async fn get_element_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElement>>;
    async fn upsert_collection_element(&self, new_element: &NewCollectionElement) -> Result<()>;
    async fn delete_collection_element(&self, element_id: &Id<CollectionElement>) -> Result<()>;
    async fn delete_element_by_id(&self, id: &Id<CollectionElement>) -> Result<()>;

    // CollectionElementInfo操作
    async fn upsert_collection_element_info(&self, info: &NewCollectionElementInfo) -> Result<()>;
    async fn get_element_info_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementInfo>>;
    async fn get_not_registered_info_element_ids(&self) -> Result<Vec<Id<CollectionElement>>>;
    async fn get_brandname_and_rubies(&self) -> Result<Vec<(String, String)>>;
    async fn get_element_ids_by_is_nukige(
        &self,
        is_nukige: bool,
    ) -> Result<Vec<Id<CollectionElement>>>;
    async fn get_element_ids_by_brandnames(
        &self,
        brandnames: &Vec<String>,
    ) -> Result<Vec<Id<CollectionElement>>>;
    async fn get_element_ids_by_sellday(
        &self,
        since: &str,
        until: &str,
    ) -> Result<Vec<Id<CollectionElement>>>;

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
    async fn get_element_ids_by_install_at_not_null(&self) -> Result<Vec<Id<CollectionElement>>>;

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

    // その他のユーティリティ操作
    async fn remove_conflict_maps(&self) -> Result<()>;
}
