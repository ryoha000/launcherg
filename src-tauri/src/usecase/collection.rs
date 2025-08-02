use std::{fs, sync::Arc};

use chrono::Local;
use derive_new::new;
use tauri::AppHandle;

use super::error::UseCaseError;
use crate::{
    domain::{
        collection::{
            CollectionElement, NewCollectionElement, ScannedGameElement, 
            NewCollectionElementDLStore, CollectionElementDLStore, DLStoreType, 
            NewCollectionElementPaths
        },
        file::{get_icon_path, get_thumbnail_path, save_thumbnail},
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

    // スクレイピング情報を保存
    pub async fn upsert_collection_element_info(
        &self,
        info: &crate::domain::collection::NewCollectionElementInfo,
    ) -> anyhow::Result<()> {
        self.repositories
            .collection_repository()
            .upsert_collection_element_info(info)
            .await?;
        Ok(())
    }

    // 関連データを含むコレクション要素を作成
    pub async fn create_collection_element(
        &self,
        element: &ScannedGameElement,
    ) -> anyhow::Result<()> {
        use crate::domain::collection::{
            NewCollectionElement, NewCollectionElementInstall, NewCollectionElementPaths,
        };

        // 1. 基本要素を作成
        let new_element = NewCollectionElement::new(element.id.clone());
        self.upsert_collection_element(&new_element).await?;

        // 2. スクレイピング情報は初期登録時には作成しない
        // （後でregisterCollectionElementDetailsから取得される）

        // 3. パス情報を保存
        if element.exe_path.is_some() || element.lnk_path.is_some() {
            let new_paths = NewCollectionElementPaths::new(
                element.id.clone(),
                element.exe_path.clone(),
                element.lnk_path.clone(),
            );
            self.repositories
                .collection_repository()
                .upsert_collection_element_paths(&new_paths)
                .await?;
        }

        // 4. インストール情報を保存
        if let Some(install_time) = element.install_at {
            let new_install = NewCollectionElementInstall::new(element.id.clone(), install_time);
            self.repositories
                .collection_repository()
                .upsert_collection_element_install(&new_install)
                .await?;
        }

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

    // 関連データ付きコレクション要素リストを一括保存
    pub async fn upsert_collection_elements(
        &self,
        source: &Vec<ScannedGameElement>,
    ) -> anyhow::Result<()> {
        for element in source.iter() {
            self.create_collection_element(element).await?;
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

        let paths = self
            .repositories
            .collection_repository()
            .get_element_paths_by_element_id(id)
            .await?;

        let icon_path = if let Some(paths) = paths {
            if let Some(lnk_path) = paths.lnk_path {
                // lnkファイルからメタデータを取得してアイコンパスを決定
                use crate::domain::file::get_lnk_metadatas;
                let metadatas = get_lnk_metadatas(vec![lnk_path.as_str()])?;
                let metadata = metadatas
                    .get(lnk_path.as_str())
                    .ok_or(anyhow::anyhow!("metadata cannot get"))?;
                if metadata.icon.to_lowercase().ends_with("ico") {
                    println!("icon is ico");
                    metadata.icon.clone()
                } else {
                    metadata.path.clone()
                }
            } else if let Some(exe_path) = paths.exe_path {
                exe_path
            } else {
                eprintln!("lnk_path and exe_path are None");
                return Ok(());
            }
        } else {
            eprintln!("No paths found for element {}", id.value);
            return Ok(());
        };

        use crate::domain::file::save_icon_to_png;
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
            .get_not_registered_info_element_ids()
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

    // DL版ゲーム管理機能
    pub async fn register_dl_store_game(
        &self,
        store_type: DLStoreType,
        store_id: String,
        erogamescape_id: i32,
        purchase_url: String,
    ) -> anyhow::Result<Id<CollectionElement>> {
        let collection_element_id = Id::new(erogamescape_id);

        // 既存のDL版ゲーム情報を検索
        if let Some(_existing) = self.repositories
            .collection_repository()
            .get_element_dl_store_by_store_id(&store_id, &store_type)
            .await? 
        {
            return Err(anyhow::anyhow!("このストアIDは既に登録されています"));
        }

        // collection_elementが存在するかチェック
        let element_exists = self.repositories
            .collection_repository()
            .get_element_by_element_id(&collection_element_id)
            .await
            .is_ok();

        if !element_exists {
            // collection_elementを作成
            let new_element = NewCollectionElement::new(collection_element_id.clone());
            self.repositories
                .collection_repository()
                .upsert_collection_element(&new_element)
                .await?;
        }

        // DL版情報を登録
        let store_name = match store_type {
            DLStoreType::DMM => "DMM Games",
            DLStoreType::DLSite => "DLsite",
        };

        let dl_store = NewCollectionElementDLStore::new(
            collection_element_id.clone(),
            store_id,
            store_type,
            store_name.to_string(),
            purchase_url,
            true, // is_owned
            None, // purchase_date (現在は手動登録のため不明)
        );

        self.repositories
            .collection_repository()
            .upsert_collection_element_dl_store(&dl_store)
            .await?;

        Ok(collection_element_id)
    }

    pub async fn link_installed_game(
        &self,
        collection_element_id: Id<CollectionElement>,
        exe_path: String,
    ) -> anyhow::Result<()> {
        let paths = NewCollectionElementPaths::new(
            collection_element_id.clone(),
            Some(exe_path),
            None, // lnk_path
        );

        self.repositories
            .collection_repository()
            .upsert_collection_element_paths(&paths)
            .await?;

        Ok(())
    }

    pub async fn get_uninstalled_owned_games(&self) -> anyhow::Result<Vec<CollectionElement>> {
        self.repositories
            .collection_repository()
            .get_uninstalled_owned_games()
            .await
    }

    pub async fn update_dl_store_ownership(
        &self,
        id: Id<CollectionElementDLStore>,
        is_owned: bool,
    ) -> anyhow::Result<()> {
        let mut dl_store = self.repositories
            .collection_repository()
            .get_element_dl_store_by_element_id(&Id::new(id.value))
            .await?
            .ok_or(anyhow::anyhow!("DL store not found"))?;

        dl_store.is_owned = is_owned;
        
        self.repositories
            .collection_repository()
            .update_collection_element_dl_store(&dl_store)
            .await?;

        Ok(())
    }

}
