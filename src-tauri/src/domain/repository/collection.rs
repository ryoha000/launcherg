use crate::domain::collection::{Collection, CollectionElement, CollectionID, NewCollection};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CollectionRepository {
    async fn get_by_name(&self, name: String) -> Result<Option<Collection>>;
    async fn get_all(&self) -> Result<Vec<Collection>>;
    async fn get_elements_by_id(&self, id: CollectionID) -> Result<Vec<CollectionElement>>;
    async fn create(&self, new: NewCollection) -> Result<Collection>;
    // async fn add_elements(&self, name: String) -> Result<Collection>;
    // async fn remove_elements(&self, name: String) -> Result<Collection>;
}
