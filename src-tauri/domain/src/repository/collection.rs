use crate::{
    collection::{
        CollectionElement, CollectionElementErogamescape, CollectionElementInfo,
        CollectionElementInstall, CollectionElementLike, CollectionElementPaths,
        CollectionElementPlay, CollectionElementThumbnail, NewCollectionElement,
        NewCollectionElementInfo, NewCollectionElementInstall, NewCollectionElementLike,
        NewCollectionElementPaths, NewCollectionElementPlay, NewCollectionElementThumbnail,
    },
    works::Work,
    Id,
};
use anyhow::Result;
use chrono::{DateTime, Local};

#[trait_variant::make(Send)]
#[mockall::automock]
pub trait CollectionRepository {
    async fn get_all_elements(&mut self) -> Result<Vec<CollectionElement>>;
    async fn get_element_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElement>>;
    async fn upsert_collection_element(&mut self, new_element: &NewCollectionElement)
        -> Result<()>;
    async fn delete_collection_element(&mut self, element_id: &Id<CollectionElement>)
        -> Result<()>;

    // 既存の要素に対して名称のみを更新する（非 upsert）
    async fn update_collection_element_gamename_by_id(
        &mut self,
        id: &Id<CollectionElement>,
        gamename: &str,
    ) -> Result<()>;

    async fn upsert_collection_element_info(
        &mut self,
        info: &NewCollectionElementInfo,
    ) -> Result<()>;
    async fn get_element_info_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementInfo>>;
    async fn get_not_registered_info_element_ids(&mut self) -> Result<Vec<Id<CollectionElement>>>;

    async fn upsert_collection_element_paths(
        &mut self,
        paths: &NewCollectionElementPaths,
    ) -> Result<()>;
    async fn get_element_paths_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementPaths>>;

    async fn upsert_collection_element_install(
        &mut self,
        install: &NewCollectionElementInstall,
    ) -> Result<()>;
    async fn get_element_install_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementInstall>>;

    async fn upsert_collection_element_play(
        &mut self,
        play: &NewCollectionElementPlay,
    ) -> Result<()>;
    async fn get_element_play_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementPlay>>;
    async fn update_element_last_play_at_by_id(
        &mut self,
        id: &Id<CollectionElement>,
        last_play_at: DateTime<Local>,
    ) -> Result<()>;

    async fn upsert_collection_element_like(
        &mut self,
        like: &NewCollectionElementLike,
    ) -> Result<()>;
    async fn delete_collection_element_like_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<()>;
    async fn get_element_like_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementLike>>;
    async fn update_element_like_at_by_id(
        &mut self,
        id: &Id<CollectionElement>,
        like_at: Option<DateTime<Local>>,
    ) -> Result<()>;

    async fn upsert_collection_element_thumbnail(
        &mut self,
        thumbnail: &NewCollectionElementThumbnail,
    ) -> Result<()>;
    async fn get_element_thumbnail_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementThumbnail>>;
    async fn upsert_collection_element_thumbnail_size(
        &mut self,
        id: &Id<CollectionElement>,
        width: i32,
        height: i32,
    ) -> Result<()>;
    async fn get_null_thumbnail_size_element_ids(&mut self) -> Result<Vec<Id<CollectionElement>>>;

    async fn get_element_erogamescape_by_element_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElementErogamescape>>;

    async fn get_collection_id_by_erogamescape_id(
        &mut self,
        erogamescape_id: i32,
    ) -> Result<Option<Id<CollectionElement>>>;
    async fn get_collection_ids_by_erogamescape_ids(
        &mut self,
        erogamescape_ids: &[i32],
    ) -> Result<Vec<(i32, Id<CollectionElement>)>>;
    async fn upsert_erogamescape_map(
        &mut self,
        collection_element_id: &Id<CollectionElement>,
        erogamescape_id: i32,
    ) -> Result<()>;

    async fn allocate_new_collection_element_id(
        &mut self,
        gamename: &str,
    ) -> Result<Id<CollectionElement>>;
    async fn get_erogamescape_id_by_collection_id(
        &mut self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<i32>>;

    async fn get_collection_ids_by_work_ids(
        &mut self,
        work_ids: &[Id<Work>],
    ) -> Result<Vec<(Id<Work>, Id<CollectionElement>)>>;
    async fn upsert_work_mapping(
        &mut self,
        collection_element_id: &Id<CollectionElement>,
        work_id: Id<Work>,
    ) -> Result<()>;
    // 非 upsert の挿入（既存重複時はエラー）
    async fn insert_work_mapping(
        &mut self,
        collection_element_id: &Id<CollectionElement>,
        work_id: Id<Work>,
    ) -> Result<()>;

    async fn get_work_ids_by_collection_ids(
        &mut self,
        collection_element_ids: &[Id<CollectionElement>],
    ) -> Result<Vec<(Id<CollectionElement>, Id<Work>)>>;
}
