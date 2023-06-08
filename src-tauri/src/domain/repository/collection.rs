use crate::domain::{
    collection::{
        Collection, CollectionElement, NewCollection, NewCollectionElement,
        NewCollectionElementDetail, UpdateCollection,
    },
    Id,
};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Local};

#[async_trait]
pub trait CollectionRepository {
    async fn get(&self, id: &Id<Collection>) -> Result<Option<Collection>>;
    async fn get_by_name(&self, name: String) -> Result<Option<Collection>>;
    async fn get_all(&self) -> Result<Vec<Collection>>;
    async fn get_elements_by_id(&self, id: &Id<Collection>) -> Result<Vec<CollectionElement>>;
    async fn get_element_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> Result<Option<CollectionElement>>;
    async fn create(&self, new: NewCollection) -> Result<Collection>;
    async fn update(&self, src: UpdateCollection) -> Result<()>;
    async fn delete(&self, id: &Id<Collection>) -> Result<()>;
    async fn create_collection_elements(
        &self,
        new_elements: Vec<NewCollectionElement>,
    ) -> Result<()>;
    async fn upsert_collection_element(&self, new_elements: &NewCollectionElement) -> Result<()>;
    async fn add_elements_by_id(
        &self,
        collection_id: &Id<Collection>,
        collection_element_ids: &Vec<Id<CollectionElement>>,
    ) -> Result<()>;
    async fn remove_elements_by_id(
        &self,
        collection_id: &Id<Collection>,
        collection_element_ids: &Vec<Id<CollectionElement>>,
    ) -> Result<()>;
    async fn remove_conflict_maps(&self) -> Result<()>;
    async fn delete_collection_element(&self, element_id: &Id<CollectionElement>) -> Result<()>;

    async fn get_not_registered_detail_element_ids(&self) -> Result<Vec<Id<CollectionElement>>>;
    async fn create_element_details(&self, details: Vec<NewCollectionElementDetail>) -> Result<()>;
    async fn get_brandname_and_rubies(&self) -> Result<Vec<(String, String)>>;

    async fn get_element_ids_by_is_nukige(
        &self,
        is_nukige: bool,
    ) -> Result<Vec<Id<CollectionElement>>>;
    async fn get_element_ids_by_install_at_not_null(&self) -> Result<Vec<Id<CollectionElement>>>;
    async fn get_element_ids_by_brandnames(
        &self,
        brandnames: &Vec<String>,
    ) -> Result<Vec<Id<CollectionElement>>>;
    async fn get_element_ids_by_sellday(
        &self,
        since: &str,
        until: &str,
    ) -> Result<Vec<Id<CollectionElement>>>;

    async fn update_element_last_play_at_by_id(
        &self,
        id: &Id<CollectionElement>,
        last_play_at: DateTime<Local>,
    ) -> Result<()>;
}
