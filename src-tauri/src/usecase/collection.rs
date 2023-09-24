use std::{collections::HashSet, fs, sync::Arc};

use chrono::Local;
use derive_new::new;

use super::{error::UseCaseError, models::collection::CreateCollection};
use crate::{
    domain::{
        collection::{
            Collection, CollectionElement, NewCollectionElement, NewCollectionElementDetail,
            UpdateCollection,
        },
        file::{get_icon_path, get_lnk_metadatas, save_icon_to_png, save_thumbnail},
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

    pub async fn save_element_icon(&self, element: &NewCollectionElement) -> anyhow::Result<()> {
        let id = &element.id;
        let icon_path;
        if let Some(lnk_path) = element.lnk_path.clone() {
            let metadatas = get_lnk_metadatas(vec![lnk_path.as_str()])?;
            let metadata = metadatas
                .get(lnk_path.as_str())
                .ok_or(anyhow::anyhow!("metadata cannot get"))?;
            if metadata.icon.to_lowercase().ends_with("ico") {
                println!("icon is ico");
                icon_path = metadata.icon.clone();
            } else {
                icon_path = metadata.path.clone();
            }
        } else if let Some(exe_path) = element.exe_path.clone() {
            icon_path = exe_path;
        } else {
            eprintln!("lnk_path and exe_path are None");
            return Ok(());
        }
        Ok(save_icon_to_png(&icon_path, id)?.await??)
    }

    pub async fn save_element_thumbnail(
        &self,
        id: &Id<CollectionElement>,
        src_url: String,
    ) -> anyhow::Result<()> {
        Ok(save_thumbnail(id, src_url).await??)
    }

    pub async fn concurency_save_thumbnails(
        &self,
        args: Vec<(Id<CollectionElement>, String)>,
    ) -> anyhow::Result<()> {
        use futures::StreamExt as _;

        futures::stream::iter(args.into_iter())
            .map(|(id, url)| save_thumbnail(&id, url))
            .buffered(50)
            .map(|v| v?)
            .for_each(|v| async move {
                match v {
                    Err(e) => eprintln!("[concurency_save_thumbnails] {}", e.to_string()),
                    _ => {}
                }
            })
            .await;
        Ok(())
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

    pub async fn get_brandname_and_rubies(&self) -> anyhow::Result<Vec<(String, String)>> {
        self.repositories
            .collection_repository()
            .get_brandname_and_rubies()
            .await
    }

    pub async fn get_collection_element_ids_by_option(
        &self,
        is_nukige: bool,
        not_nukige: bool,
        is_exist_path: bool,
        brandnames: &Option<Vec<String>>,
        between: &Option<(String, String)>,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        let is_nukige_set = is_nukige.then_some(
            self.repositories
                .collection_repository()
                .get_element_ids_by_is_nukige(true)
                .await?
                .into_iter()
                .map(|v| v.value)
                .collect::<HashSet<i32>>(),
        );
        let not_nukige_set = not_nukige.then_some(
            self.repositories
                .collection_repository()
                .get_element_ids_by_is_nukige(false)
                .await?
                .into_iter()
                .map(|v| v.value)
                .collect::<HashSet<i32>>(),
        );
        let exist_path_set = (is_exist_path).then_some(
            self.repositories
                .collection_repository()
                .get_element_ids_by_install_at_not_null()
                .await?
                .into_iter()
                .map(|v| v.value)
                .collect::<HashSet<i32>>(),
        );
        let brandnames_set = match brandnames {
            Some(brandnames) => Some(
                self.repositories
                    .collection_repository()
                    .get_element_ids_by_brandnames(brandnames)
                    .await?
                    .into_iter()
                    .map(|v| v.value)
                    .collect::<HashSet<i32>>(),
            ),
            None => None,
        };
        let betwern_set = match between {
            Some((since, until)) => Some(
                self.repositories
                    .collection_repository()
                    .get_element_ids_by_sellday(since, until)
                    .await?
                    .into_iter()
                    .map(|v| v.value)
                    .collect::<HashSet<i32>>(),
            ),
            None => None,
        };

        let mut hashset_iter = vec![
            is_nukige_set,
            not_nukige_set,
            exist_path_set,
            brandnames_set,
            betwern_set,
        ]
        .into_iter()
        .filter_map(|v| v);

        let first = match hashset_iter.next() {
            Some(set) => set,
            None => return Ok(vec![]),
        };

        Ok(hashset_iter
            .fold(first, |acc, set| {
                // Find the intersection with the accumulated set and the current set
                HashSet::from_iter(acc.intersection(&set).cloned())
            })
            .into_iter()
            .map(|v| Id::new(v))
            .collect())
    }
    pub async fn update_element_last_play_at(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .update_element_last_play_at_by_id(id, Local::now())
            .await?;
        Ok(())
    }
    pub async fn update_element_like_at(
        &self,
        id: &Id<CollectionElement>,
        is_like: bool,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .update_element_like_at_by_id(id, is_like.then_some(Local::now()))
            .await?;
        Ok(())
    }
    pub async fn delete_element(&self, id: &Id<CollectionElement>) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .delete_collection_element(id)
            .await?;
        Ok(())
    }
    pub async fn get_all_elements(&self) -> anyhow::Result<Vec<CollectionElement>> {
        self.repositories
            .collection_repository()
            .get_all_elements()
            .await
    }
}
