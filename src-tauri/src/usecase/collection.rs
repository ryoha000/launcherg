use std::sync::Arc;

use derive_new::new;

use super::{error::UseCaseError, models::collection::CreateCollection};
use crate::{
    domain::{
        collection::{Collection, CollectionElement, CollectionID},
        repository::collection::CollectionRepository,
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

    pub async fn get_all_collections(&self) -> anyhow::Result<Vec<Collection>> {
        self.repositories.collection_repository().get_all().await
    }

    pub async fn get_elements_by_id(
        &self,
        id: CollectionID,
    ) -> anyhow::Result<Vec<CollectionElement>> {
        self.repositories
            .collection_repository()
            .get_elements_by_id(id)
            .await
    }
}
