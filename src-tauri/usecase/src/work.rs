use std::sync::Arc;

use derive_new::new;
use domain::repository::save_image_queue::ImageSaveQueueRepository as _;
use domain::repository::work_parent_packs::WorkParentPacksRepository as _;
use domain::repository::works::DmmWorkRepository as _;
use domain::repository::{
    manager::RepositoryManager, work_like::WorkLikeRepository,
    work_lnk::WorkLnkRepository, works::WorkRepository, RepositoriesExt,
};
use domain::service::save_path_resolver::SavePathResolver;
use domain::windows::{shell_link::ShellLink as ShellLinkTrait, WindowsExt};
use domain::works::WorkDetails;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub enum RegisterWorkPathInput {
    Exe { exe_path: String },
    Lnk { lnk_path: String },
}

#[derive(new)]
pub struct WorkUseCase<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    windows: Arc<W>,
    resolver: Arc<dyn SavePathResolver>,
    #[new(default)]
    _marker: PhantomData<R>,
}

impl<M, R, W> WorkUseCase<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    pub async fn list_all_details(&self) -> anyhow::Result<Vec<WorkDetails>> {
        self.manager
            .run(|repos| Box::pin(async move { repos.work().list_all_details().await }))
            .await
    }


    pub async fn find_details_by_work_id(&self, work_id: String) -> anyhow::Result<Option<WorkDetails>> {
        self.manager
            .run(|repos| {
                Box::pin(async move { repos.work().find_details_by_work_id(domain::StrId::new(work_id)).await })
            })
            .await
    }

    pub async fn list_work_lnks(&self, work_id: String) -> anyhow::Result<Vec<(i32, String)>> {
        let wid = domain::StrId::new(work_id);
        let list = self
            .manager
            .run(|repos| Box::pin(async move { repos.work_lnk().list_by_work_id(wid).await }))
            .await?;
        Ok(list.into_iter().map(|e| (e.id.value, e.lnk_path)).collect())
    }

    pub async fn launch_work(
        &self,
        is_run_as_admin: bool,
        work_lnk_id: i32,
    ) -> anyhow::Result<Option<u32>> {
        let lnk = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    repos
                        .work_lnk()
                        .find_by_id(domain::Id::new(work_lnk_id))
                        .await
                })
            })
            .await?;

        let lnk = lnk.ok_or(anyhow::anyhow!(format!(
            "work_lnk not found: {}",
            work_lnk_id
        )))?;
        // ShellLink 経由で .lnk を実行
        let pid = self
            .windows
            .shell_link()
            .execute_lnk(&lnk.lnk_path, is_run_as_admin)?;

        // last_play_at を更新（起動成功時のみ）
        if pid.is_some() {
            let work_id = lnk.work_id;
            let _ = self
                .manager
                .run(|repos| {
                    Box::pin(async move {
                        repos
                            .work()
                            .update_last_play_at_by_work_id(work_id, chrono::Local::now())
                            .await
                    })
                })
                .await;
        }

        Ok(pid)
    }

    pub async fn get_parent_dmm_pack_work_id(&self, work_id: String) -> anyhow::Result<Option<String>> {
        let wid = domain::StrId::new(work_id);
        let parent = self
            .manager
            .run(|repos| {
                Box::pin(async move { repos.work_parent_packs().find_parent_id(wid).await })
            })
            .await?;
        Ok(parent.map(|p| p.value))
    }

    pub async fn get_dmm_work_by_work_id(
        &self,
        work_id: String,
    ) -> anyhow::Result<Option<domain::works::DmmWork>> {
        let wid = domain::StrId::new(work_id);
        self.manager
            .run(|repos| Box::pin(async move { repos.dmm_work().find_by_work_id(wid).await }))
            .await
    }

    pub async fn update_like(&self, work_id: String, is_like: bool) -> anyhow::Result<()> {
        let wid = domain::StrId::new(work_id);
        self.manager
            .run(|repos| {
                Box::pin(async move {
                    repos
                        .work_like()
                        .update_like_at_by_work_id(wid, is_like.then_some(chrono::Local::now()))
                        .await
                })
            })
            .await
    }

    pub async fn delete_work(&self, work_id: String) -> anyhow::Result<()> {
        let wid = domain::StrId::new(work_id);
        self.manager
            .run(|repos| Box::pin(async move { repos.work().delete(wid).await }))
            .await
    }

    pub async fn register_work_from_input(
        &self,
        erogamescape_id: i32,
        title: String,
        thumbnail_url: String,
        path: RegisterWorkPathInput,
    ) -> anyhow::Result<()> {
        let resolver = self.resolver.clone();
        self.manager
            .run_in_transaction(|repos| {
                let resolver = resolver.clone();
                let title = title.clone();
                let thumbnail_url = thumbnail_url.clone();
                let path = path.clone();
                let erogamescape_id = erogamescape_id;
                Box::pin(async move {
                    // 1) Work を EGS ID で検索。なければ新規作成
                    let mut work_repo = repos.work();
                    let existing = work_repo
                        .find_work_ids_by_erogamescape_ids(&[erogamescape_id])
                        .await?;
                    let (work_id, is_new) = match existing.into_iter().next() {
                        Some((_egs, id)) => (id, false),
                        None => (
                            work_repo
                                .upsert(&domain::works::NewWork::new(title.clone()))
                                .await?,
                            true,
                        ),
                    };

                    // 2) EGS マップは新規作成時のみ upsert
                    if is_new {
                        repos
                            .work()
                            .upsert_erogamescape_map(work_id.clone(), erogamescape_id)
                            .await?;
                    }

                    // 3) パス登録（enum でパターンマッチ）
                    match path {
                        RegisterWorkPathInput::Lnk { lnk_path } => {
                            // そのまま登録（既存ファイルパスを採用）
                            let _ = repos
                                .work_lnk()
                                .insert(&domain::repository::work_lnk::NewWorkLnk {
                                    work_id: work_id.clone(),
                                    lnk_path,
                                })
                                .await?;
                        }
                        RegisterWorkPathInput::Exe { exe_path } => {
                            // .lnk を生成して登録
                            let dst = resolver.lnk_new_path(&work_id.value);
                            if let Some(parent) = std::path::Path::new(&dst).parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            let req = domain::windows::shell_link::CreateShortcutRequest {
                                target_path: exe_path,
                                dest_lnk_path: dst.clone(),
                                working_dir: None,
                                arguments: None,
                                icon_path: None,
                            };
                            let _ = self.windows.shell_link().create_bulk(vec![req]);
                            let _ = repos
                                .work_lnk()
                                .insert(&domain::repository::work_lnk::NewWorkLnk {
                                    work_id: work_id.clone(),
                                    lnk_path: dst,
                                })
                                .await?;
                        }
                    }

                    // 4) サムネイルをキュー投入
                    if !thumbnail_url.is_empty() {
                        let thumb_dst = resolver.thumbnail_png_path(&work_id.value);
                        let _ = repos
                            .image_queue()
                            .enqueue(
                                &thumbnail_url,
                                domain::save_image_queue::ImageSrcType::Url,
                                &thumb_dst,
                                domain::save_image_queue::ImagePreprocess::ResizeForWidth400,
                            )
                            .await;
                    }

                    Ok::<(), anyhow::Error>(())
                })
            })
            .await
    }
}
