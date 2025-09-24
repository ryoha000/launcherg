use std::{fs, sync::Arc};

use chrono::Local;
use derive_new::new;
use domain::{service::save_path_resolver::SavePathResolver, thumbnail::ThumbnailService};

use super::error::UseCaseError;
use domain::repository::work_lnk::{NewWorkLnk, WorkLnkRepository};
use domain::repository::works::WorkRepository;
use domain::repository::{
    collection::CollectionRepository, manager::RepositoryManager, RepositoriesExt,
};
use domain::windows::shell_link::{CreateShortcutRequest, ShellLink as _};
use domain::windows::WindowsExt;
use domain::{
    collection::{CollectionElement, NewCollectionElement, ScannedGameElement},
    Id,
};
use std::marker::PhantomData;

#[derive(new)]
pub struct CollectionUseCase<M, R, TS, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    TS: ThumbnailService,
    W: WindowsExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    resolver: Arc<dyn SavePathResolver>,
    thumbnail_service: Arc<TS>,
    windows: Arc<W>,
    #[new(default)]
    _marker: PhantomData<R>,
}

impl<M, R, TS, W> CollectionUseCase<M, R, TS, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    TS: ThumbnailService,
    W: WindowsExt + Send + Sync + 'static,
{
    pub async fn migrate_collection_paths_to_work_lnks(&self) -> anyhow::Result<()> {
        use domain::Id;
        // 1) collection_elements と work の対応を取得
        // 2) collection_element_paths の exe/lnk を lnks ディレクトリへ集約
        // 3) work_lnks へ INSERT（重複は UNIQUE で自然排他）

        // get all collection elements with paths and resolve work_id via work_collection_elements
        let rows = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    let mut repo = repos.collection();
                    // 旧APIで全件取得し paths を拾う
                    Ok::<Vec<domain::collection::CollectionElement>, anyhow::Error>(
                        repo.get_all_elements().await?,
                    )
                })
            })
            .await?;

        let mut bulk_requests: Vec<CreateShortcutRequest> = Vec::new();
        let mut to_insert: Vec<(Id<domain::works::Work>, String)> = Vec::new();

        for element in rows.into_iter() {
            let ce_id = element.id.clone();
            // work_id を解決
            let work_details = self
                .manager
                .run(|repos| {
                    Box::pin(async move {
                        let mut repo = repos.work();
                        Ok::<Option<domain::works::WorkDetails>, anyhow::Error>(
                            repo.find_details_by_collection_element_id(ce_id.clone())
                                .await?,
                        )
                    })
                })
                .await?;
            let Some(details) = work_details else {
                continue;
            };
            let work_id: Id<domain::works::Work> = details.work.id;

            if let Some(paths) = element.paths.clone() {
                if let Some(lnk) = paths.lnk_path.as_ref() {
                    let dst = self.resolver.lnk_new_path(work_id.value);
                    let _ = std::fs::create_dir_all(std::path::Path::new(&dst).parent().unwrap());
                    let _ = std::fs::copy(&lnk, &dst);
                    to_insert.push((work_id.clone(), dst));
                } else if let Some(exe) = paths.exe_path.as_ref() {
                    let dst = self.resolver.lnk_new_path(work_id.value);
                    let _ = std::fs::create_dir_all(std::path::Path::new(&dst).parent().unwrap());
                    bulk_requests.push(CreateShortcutRequest {
                        target_path: exe.clone(),
                        dest_lnk_path: dst.clone(),
                        working_dir: None,
                        arguments: None,
                        icon_path: None,
                    });
                    to_insert.push((work_id.clone(), dst));
                }
            }
        }
        // 一括作成（存在する場合のみ）
        if !bulk_requests.is_empty() {
            let _ = self.windows.shell_link().create_bulk(bulk_requests)?;
        }
        // 生成・コピー済み lnks を登録
        for (wid, lnk_path) in to_insert.into_iter() {
            let _ = self
                .manager
                .run(|repos| {
                    Box::pin(async move {
                        let mut repo = repos.work_lnk();
                        let _ = repo
                            .insert(&NewWorkLnk {
                                work_id: wid,
                                lnk_path,
                            })
                            .await;
                        Ok::<(), anyhow::Error>(())
                    })
                })
                .await;
        }
        Ok(())
    }

    pub async fn upsert_collection_element(
        &self,
        source: &NewCollectionElement,
    ) -> anyhow::Result<()> {
        self.manager
            .run(|repos| {
                Box::pin(async move { repos.collection().upsert_collection_element(source).await })
            })
            .await?;
        Ok(())
    }

    // 関連データを含むコレクション要素を作成
    pub async fn create_collection_element(
        &self,
        element: &ScannedGameElement,
    ) -> anyhow::Result<Id<CollectionElement>> {
        use domain::collection::{NewCollectionElement, NewCollectionElementInstall};

        // 1. erogamescape_id から collection_element_id を解決/作成
        let resolved_id = {
            let egid = element.erogamescape_id;
            let name = element.gamename.clone();
            self.manager
                .run(|repos| {
                    Box::pin(async move {
                        let mut repo = repos.collection();
                        if let Some(mapped) =
                            repo.get_collection_id_by_erogamescape_id(egid).await?
                        {
                            let new_element =
                                NewCollectionElement::new(mapped.clone(), name.clone());
                            repo.upsert_collection_element(&new_element).await?;
                            Ok(mapped)
                        } else {
                            let id = repo.allocate_new_collection_element_id(&name).await?;
                            let _ = repo.upsert_erogamescape_map(&id, egid).await;
                            Ok(id)
                        }
                    })
                })
                .await?
        };

        // 2. スクレイピング情報は初期登録時には作成しない
        // （後でregisterCollectionElementDetailsから取得される）

        // 3. パス情報を保存 → work_lnks 登録へ変更
        if element.exe_path.is_some() || element.lnk_path.is_some() {
            // work_id 解決
            let cid = resolved_id.clone();
            let work_details = self
                .manager
                .run(|repos| {
                    Box::pin(async move {
                        let mut repo = repos.work();
                        Ok::<Option<domain::works::WorkDetails>, anyhow::Error>(
                            repo.find_details_by_collection_element_id(cid).await?,
                        )
                    })
                })
                .await?;
            if let Some(details) = work_details {
                let work_id = details.work.id;
                let lnk_save_path = if let Some(lnk) = element.lnk_path.as_ref() {
                    let dst = self.resolver.lnk_new_path(work_id.value);
                    let _ = std::fs::create_dir_all(std::path::Path::new(&dst).parent().unwrap());
                    let _ = std::fs::copy(&lnk, &dst);
                    Some(dst)
                } else if let Some(exe) = element.exe_path.as_ref() {
                    let dst = self.resolver.lnk_new_path(work_id.value);
                    let _ = std::fs::create_dir_all(std::path::Path::new(&dst).parent().unwrap());
                    let _ = self
                        .windows
                        .shell_link()
                        .create_bulk(vec![CreateShortcutRequest {
                            target_path: exe.clone(),
                            dest_lnk_path: dst.clone(),
                            working_dir: None,
                            arguments: None,
                            icon_path: None,
                        }]);
                    Some(dst)
                } else {
                    None
                };
                if let Some(lnk_path) = lnk_save_path {
                    let work_id_clone = work_id.clone();
                    self.manager
                        .run(|repos| {
                            Box::pin(async move {
                                repos
                                    .work_lnk()
                                    .insert(&NewWorkLnk {
                                        work_id: work_id_clone,
                                        lnk_path,
                                    })
                                    .await
                                    .map(|_| ())
                            })
                        })
                        .await?;
                }
            }
        }

        // 4. インストール情報を保存
        if let Some(install_time) = element.install_at {
            let new_install = NewCollectionElementInstall::new(resolved_id.clone(), install_time);
            self.manager
                .run(|repos| {
                    Box::pin(async move {
                        repos
                            .collection()
                            .upsert_collection_element_install(&new_install)
                            .await
                    })
                })
                .await?;
        }

        Ok(resolved_id)
    }

    pub async fn upsert_collection_element_thumbnail_size(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let thumbnail_path = self.resolver.thumbnail_png_path(id.value);
        match image::image_dimensions(thumbnail_path) {
            Ok((width, height)) => {
                self.manager
                    .run(|repos| {
                        Box::pin(async move {
                            repos
                                .collection()
                                .upsert_collection_element_thumbnail_size(
                                    id,
                                    width as i32,
                                    height as i32,
                                )
                                .await
                        })
                    })
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
        ids: Vec<Id<CollectionElement>>,
    ) -> anyhow::Result<()> {
        use futures::StreamExt as _;

        futures::stream::iter(ids.into_iter())
            .map(move |id| async move { self.upsert_collection_element_thumbnail_size(&id).await })
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
        let row = self
            .manager
            .run(|repos| {
                Box::pin(async move { repos.collection().get_element_by_element_id(id).await })
            })
            .await?;
        Ok(row.ok_or(UseCaseError::CollectionElementIsNotFound)?)
    }

    pub async fn update_collection_element_icon(
        &self,
        id: &Id<CollectionElement>,
        path: String,
    ) -> anyhow::Result<()> {
        let save_icon_path = self.resolver.icon_png_path(id.value);
        fs::copy(path, save_icon_path)?;
        Ok(())
    }

    pub async fn save_element_icon(&self, id: &Id<CollectionElement>) -> anyhow::Result<()> {
        // collection_element_id -> work_id 解決
        let cid = id.clone();
        let work_details = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    let mut repo = repos.work();
                    Ok::<Option<domain::works::WorkDetails>, anyhow::Error>(
                        repo.find_details_by_collection_element_id(cid).await?,
                    )
                })
            })
            .await?;
        let Some(details) = work_details else {
            return Ok(());
        };

        // work_lnks から代表 lnk を取得
        let work_id = details.work.id;
        let lnks = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    let mut repo = repos.work_lnk();
                    Ok::<Vec<domain::repository::work_lnk::WorkLnk>, anyhow::Error>(
                        repo.list_by_work_id(work_id).await?,
                    )
                })
            })
            .await?;
        let Some(primary) = lnks.into_iter().next() else {
            return Ok(());
        };

        // lnk メタからアイコン保存
        let lnk_path = primary.lnk_path;
        let metadatas = self
            .windows
            .shell_link()
            .get_lnk_metadatas(vec![lnk_path.clone()])?;
        if let Some(metadata) = metadatas.get(&lnk_path) {
            let dst = self.resolver.icon_png_path(id.value);
            if metadata.icon.to_lowercase().ends_with("ico") {
                let _ = domain::file::save_ico_to_png(&metadata.icon, &dst)?.await?;
            } else {
                let _ = std::fs::copy(&metadata.path, &dst);
            }
        }
        Ok(())
    }

    pub async fn save_element_thumbnail(
        &self,
        id: &Id<CollectionElement>,
        src_url: String,
    ) -> anyhow::Result<()> {
        self.thumbnail_service.save_thumbnail(id, &src_url).await
    }

    pub async fn delete_collection_element_by_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        let existed = self
            .manager
            .run(|repos| {
                Box::pin(async move { repos.collection().get_element_by_element_id(id).await })
            })
            .await?;
        if existed.is_none() {
            return Err(UseCaseError::CollectionElementIsNotFound.into());
        }
        self.manager
            .run(|repos| {
                Box::pin(async move { repos.collection().delete_collection_element(id).await })
            })
            .await
    }

    pub async fn get_not_registered_detail_element_ids(
        &self,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        // 旧APIは廃止。互換のため空配列を返す（呼び出し元は現状なし）
        Ok(Vec::new())
    }

    pub async fn update_element_last_play_at(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<()> {
        self.manager
            .run(|repos| {
                Box::pin(async move {
                    repos
                        .collection()
                        .update_element_last_play_at_by_id(id, Local::now())
                        .await
                })
            })
            .await?;
        Ok(())
    }
    pub async fn update_element_like_at(
        &self,
        id: &Id<CollectionElement>,
        is_like: bool,
    ) -> anyhow::Result<()> {
        self.manager
            .run(|repos| {
                Box::pin(async move {
                    repos
                        .collection()
                        .update_element_like_at_by_id(id, is_like.then_some(Local::now()))
                        .await
                })
            })
            .await?;
        Ok(())
    }
    pub async fn get_all_elements(&self) -> anyhow::Result<Vec<CollectionElement>> {
        let null_size_ids = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    repos
                        .collection()
                        .get_null_thumbnail_size_element_ids()
                        .await
                })
            })
            .await?;
        self.concurency_upsert_collection_element_thumbnail_size(null_size_ids)
            .await?;

        self.manager
            .run(|repos| Box::pin(async move { repos.collection().get_all_elements().await }))
            .await
    }

    pub async fn link_installed_game(
        &self,
        collection_element_id: Id<CollectionElement>,
        exe_path: String,
    ) -> anyhow::Result<()> {
        let work_details = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    let mut repo = repos.work();
                    Ok::<Option<domain::works::WorkDetails>, anyhow::Error>(
                        repo.find_details_by_collection_element_id(collection_element_id.clone())
                            .await?,
                    )
                })
            })
            .await?;
        if let Some(details) = work_details {
            let work_id = details.work.id;
            let dst = self.resolver.lnk_new_path(work_id.value);
            let _ = std::fs::create_dir_all(std::path::Path::new(&dst).parent().unwrap());
            let _ = self
                .windows
                .shell_link()
                .create_bulk(vec![CreateShortcutRequest {
                    target_path: exe_path.clone(),
                    dest_lnk_path: dst.clone(),
                    working_dir: None,
                    arguments: None,
                    icon_path: None,
                }]);
            let wid = work_id.clone();
            self.manager
                .run(|repos| {
                    Box::pin(async move {
                        repos
                            .work_lnk()
                            .insert(&NewWorkLnk {
                                work_id: wid,
                                lnk_path: dst,
                            })
                            .await
                            .map(|_| ())
                    })
                })
                .await?;
        }

        Ok(())
    }

    // EGS ID から collection_element_id 群を解決
    pub async fn get_collection_ids_by_erogamescape_ids(
        &self,
        erogamescape_ids: Vec<i32>,
    ) -> anyhow::Result<Vec<Id<CollectionElement>>> {
        self.manager
            .run(|repos| {
                Box::pin(async move {
                    let mut repo = repos.collection();
                    let mut ids = Vec::new();
                    for egs_id in erogamescape_ids {
                        if let Some(id) = repo.get_collection_id_by_erogamescape_id(egs_id).await? {
                            ids.push(id);
                        }
                    }
                    Ok(ids)
                })
            })
            .await
    }

    // collection_element_id -> erogamescape_id（単発）
    pub async fn get_erogamescape_id_by_collection_id(
        &self,
        id: &Id<CollectionElement>,
    ) -> anyhow::Result<Option<i32>> {
        self.manager
            .run(|repos| {
                Box::pin(async move {
                    repos
                        .collection()
                        .get_erogamescape_id_by_collection_id(id)
                        .await
                })
            })
            .await
    }
}
