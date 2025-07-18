use std::{fs, sync::Arc};

use chrono::Local;
use derive_new::new;
use tauri::AppHandle;

use super::error::UseCaseError;
use crate::{
    domain::{
        collection::{CollectionElement, NewCollectionElement, NewCollectionElementDetail},
        file::{
            get_icon_path, get_lnk_metadatas, get_thumbnail_path, save_icon_to_png, save_thumbnail,
        },
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
    pub async fn upsert_collection_element_thumbnail_size(
        &self,
        handle: &Arc<AppHandle>,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let thumbnail_path = get_thumbnail_path(handle, id);
        match image::image_dimensions(thumbnail_path) {
            Ok((width, height)) => {
                self.repositories
                    .collection_repository()
                    .upsert_collection_element_thumbnail_size(id, width as i32, height as i32)
                    .await?;
            }
            Err(e) => {
                eprintln!(
                    "[upsert_collection_element_thumbnail_size] {}",
                    e.to_string()
                );
            }
        }
        Ok(())
    }
    pub async fn concurency_upsert_collection_element_thumbnail_size(
        &self,
        handle: &Arc<AppHandle>,
        ids: Vec<Id<CollectionElement>>,
    ) -> anyhow::Result<()> {
        use futures::StreamExt as _;

        futures::stream::iter(ids.into_iter())
            .map(move |id| {
                let id = id.clone();
                let handle_cloned = handle.clone();
                async move {
                    self.upsert_collection_element_thumbnail_size(&handle_cloned, &id)
                        .await
                }
            })
            .buffered(50)
            .for_each(|v| async move {
                match v {
                    Err(e) => eprintln!(
                        "[concurency_upsert_collection_element_thumbnail_size] {}",
                        e.to_string()
                    ),
                    _ => {}
                }
            })
            .await;
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
        handle: &Arc<AppHandle>,
        id: &Id<CollectionElement>,
        path: String,
    ) -> anyhow::Result<()> {
        let save_icon_path = get_icon_path(handle, id);
        fs::copy(path, save_icon_path)?;
        Ok(())
    }

    pub async fn save_element_icon(
        &self,
        handle: &Arc<AppHandle>,
        element: &NewCollectionElement,
    ) -> anyhow::Result<()> {
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
        Ok(save_icon_to_png(handle, &icon_path, id)?.await??)
    }

    pub async fn save_element_thumbnail(
        &self,
        handle: &Arc<AppHandle>,
        id: &Id<CollectionElement>,
        src_url: String,
    ) -> anyhow::Result<()> {
        Ok(save_thumbnail(handle, id, src_url).await??)
    }

    pub async fn concurency_save_thumbnails(
        &self,
        handle: &Arc<AppHandle>,
        args: Vec<(Id<CollectionElement>, String)>,
    ) -> anyhow::Result<()> {
        use futures::StreamExt as _;

        futures::stream::iter(args.into_iter())
            .map(|(id, url)| save_thumbnail(handle, &id, url))
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
        self.repositories
            .collection_repository()
            .delete_collection_element(id)
            .await
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
    pub async fn get_all_elements(
        &self,
        handle: &Arc<AppHandle>,
    ) -> anyhow::Result<Vec<CollectionElement>> {
        let null_size_ids = self
            .repositories
            .collection_repository()
            .get_null_thumbnail_size_element_ids()
            .await?;
        self.concurency_upsert_collection_element_thumbnail_size(handle, null_size_ids)
            .await?;

        self.repositories
            .collection_repository()
            .get_all_elements()
            .await
    }
}
