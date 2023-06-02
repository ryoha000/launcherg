use std::sync::Arc;

use derive_new::new;

use super::{error::UseCaseError, models::collection::CreateCollection};
use crate::{
    domain::{
        collection::{Collection, CollectionElement, NewCollectionElement},
        repository::collection::CollectionRepository,
        Id,
    },
    infrastructure::repositoryimpl::repository::RepositoriesExt,
};

#[derive(new)]
pub struct CollectionUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> CollectionUseCase<R> {
    pub async fn create_collection(&self, source: CreateCollection) -> anyhow::Result<Collection> {
        let existed = self
            .repositories
            .collection_repository()
            .get_by_name(source.name.clone())
            .await?;
        if existed.is_some() {
            return Err(UseCaseError::CollectionIsAlreadyExist.into());
        }
        self.repositories
            .collection_repository()
            .create(source.try_into()?)
            .await
    }
    pub async fn create_collection_elements(
        &self,
        source: Vec<NewCollectionElement>,
    ) -> anyhow::Result<()> {
        Ok(self
            .repositories
            .collection_repository()
            .create_collection_elements(source)
            .await?)
    }
    pub async fn upsert_collection_elements(
        &self,
        source: &Vec<NewCollectionElement>,
    ) -> anyhow::Result<()> {
        let tasks = source.into_iter().map(|v| {
            self.repositories
                .collection_repository()
                .upsert_collection_element(v)
        });
        futures::future::try_join_all(tasks).await?;
        Ok(())
    }
    pub async fn add_collection_elements(
        &self,
        collection_id: &Id<Collection>,
        collection_element_ids: &Vec<Id<CollectionElement>>,
    ) -> anyhow::Result<()> {
        Ok(self
            .repositories
            .collection_repository()
            .add_elements_by_id(collection_id, collection_element_ids)
            .await?)
    }

    pub async fn get_all_collections(&self) -> anyhow::Result<Vec<Collection>> {
        self.repositories.collection_repository().get_all().await
    }

    pub async fn get_elements_by_id(
        &self,
        id: &Id<Collection>,
    ) -> anyhow::Result<Vec<CollectionElement>> {
        self.repositories
            .collection_repository()
            .get_elements_by_id(id)
            .await
    }
}
