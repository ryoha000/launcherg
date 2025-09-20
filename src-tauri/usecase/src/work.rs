use std::sync::Arc;

use derive_new::new;
use domain::repository::work_parent_packs::WorkParentPacksRepository as _;
use domain::repository::works::DmmWorkRepository as _;
use domain::repository::{
    collection::CollectionRepository, manager::RepositoryManager, work_lnk::WorkLnkRepository,
    works::WorkRepository, RepositoriesExt,
};
use domain::windows::{shell_link::ShellLink as ShellLinkTrait, WindowsExt};
use domain::works::WorkDetails;
use std::marker::PhantomData;

#[derive(new)]
pub struct WorkUseCase<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    windows: Arc<W>,
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

    pub async fn find_details_by_collection_element_id(
        &self,
        collection_element_id: i32,
    ) -> anyhow::Result<Option<WorkDetails>> {
        self.manager
            .run(|repos| {
                Box::pin(async move {
                    repos
                        .work()
                        .find_details_by_collection_element_id(domain::Id::new(
                            collection_element_id,
                        ))
                        .await
                })
            })
            .await
    }

    pub async fn list_work_lnks(&self, work_id: i32) -> anyhow::Result<Vec<(i32, String)>> {
        let wid = domain::Id::new(work_id);
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

        // last_play_at 更新（work_id -> collection_element_id を解決）
        let work_id_val = lnk.work_id.value;
        let _ = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    let pairs = repos
                        .collection()
                        .get_collection_ids_by_work_ids(&[domain::Id::new(work_id_val)])
                        .await?;
                    if let Some((_, ce_id)) = pairs.into_iter().next() {
                        let _ = repos
                            .collection()
                            .update_element_last_play_at_by_id(&ce_id, chrono::Local::now())
                            .await;
                    }
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await;

        Ok(pid)
    }

    pub async fn get_parent_dmm_pack_work_id(&self, work_id: i32) -> anyhow::Result<Option<i32>> {
        let wid = domain::Id::new(work_id);
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
        work_id: i32,
    ) -> anyhow::Result<Option<domain::works::DmmWork>> {
        let wid = domain::Id::new(work_id);
        self.manager
            .run(|repos| Box::pin(async move { repos.dmm_work().find_by_work_id(wid).await }))
            .await
    }
}
