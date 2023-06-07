use std::{fs, sync::Arc};

use derive_new::new;

use super::{error::UseCaseError, models::collection::CreateCollection};
use crate::{
    domain::{
        collection::{
            Collection, CollectionElement, NewCollectionElement, NewCollectionElementDetail,
            UpdateCollection,
        },
        file::{get_icon_path, save_icon_to_png},
        repository::collection::CollectionRepository,
        Id,
    },
    infrastructure::repositoryimpl::{
        migration::ONEPIECE_COLLECTION_ID, repository::RepositoriesExt,
    },
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
    pub async fn update_collection_by_id(&self, src: UpdateCollection) -> anyhow::Result<()> {
        let existed = self
            .repositories
            .collection_repository()
            .get(&src.id)
            .await?;
        if existed.is_none() {
            return Err(UseCaseError::CollectionIsNotFound.into());
        }
        Ok(self
            .repositories
            .collection_repository()
            .update(src)
            .await?)
    }
    pub async fn delete_collection_by_id(&self, id: &Id<Collection>) -> anyhow::Result<()> {
        if id.value == ONEPIECE_COLLECTION_ID {
            return Err(UseCaseError::CollectionNotPermittedToDelete.into());
        }
        let existed = self.repositories.collection_repository().get(id).await?;
        if existed.is_none() {
            return Err(UseCaseError::CollectionIsNotFound.into());
        }
        Ok(self.repositories.collection_repository().delete(id).await?)
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
    pub async fn upsert_collection_element(
        &self,
        source: &NewCollectionElement,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .upsert_collection_element(source)
            .await?;
        Ok(())
    }
    pub async fn upsert_collection_elements(
        &self,
        source: &Vec<NewCollectionElement>,
    ) -> anyhow::Result<()> {
        for v in source.into_iter() {
            self.repositories
                .collection_repository()
                .upsert_collection_element(v)
                .await?
        }
        Ok(())
    }
    pub async fn add_collection_elements(
        &self,
        collection_id: &Id<Collection>,
        collection_element_ids: &Vec<Id<CollectionElement>>,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .add_elements_by_id(collection_id, collection_element_ids)
            .await?;
        Ok(self
            .repositories
            .collection_repository()
            .remove_conflict_maps()
            .await?)
    }

    pub async fn remove_collection_elements(
        &self,
        collection_id: &Id<Collection>,
        collection_element_ids: &Vec<Id<CollectionElement>>,
    ) -> anyhow::Result<()> {
        Ok(self
            .repositories
            .collection_repository()
            .remove_elements_by_id(collection_id, collection_element_ids)
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

    pub async fn get_element_by_element_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<CollectionElement> {
        Ok(self
            .repositories
            .collection_repository()
            .get_element_by_element_id(id)
            .await?
            .ok_or(UseCaseError::CollectionElementIsNotFound)?)
    }

    pub async fn update_collection_element_icon(
        &self,
        id: &Id<CollectionElement>,
        path: String,
    ) -> anyhow::Result<()> {
        let save_icon_path = get_icon_path(id);
        fs::copy(path, save_icon_path)?;
        Ok(())
    }

    pub async fn save_element_icon(
        &self,
        path: &str,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        Ok(save_icon_to_png(path, id)?.await??)
    }

    pub async fn delete_collection_element_by_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let existed = self
            .repositories
            .collection_repository()
            .get_element_by_element_id(id)
            .await?;
        if existed.is_none() {
            return Err(UseCaseError::CollectionElementIsNotFound.into());
        }
        Ok(self
            .repositories
            .collection_repository()
            .delete_collection_element(id)
            .await?)
    }

    pub async fn get_not_registered_detail_element_ids(
        &self,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        self.repositories
            .collection_repository()
            .get_not_registered_detail_element_ids()
            .await
    }

    pub async fn create_element_details(
        &self,
        details: Vec<NewCollectionElementDetail>,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .create_element_details(details)
            .await
    }
}
