use crate::domain::{
    collection::{
        Collection, CollectionElement, NewCollection, NewCollectionElement, UpdateCollection,
    },
    Id,
};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CollectionRepository {
    async fn get(&self, id: &Id<Collection>) -> Result<Option<Collection>>;
    async fn get_by_name(&self, name: String) -> Result<Option<Collection>>;
    async fn get_all(&self) -> Result<Vec<Collection>>;
    async fn get_elements_by_id(&self, id: &Id<Collection>) -> Result<Vec<CollectionElement>>;
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
}
