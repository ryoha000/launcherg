use std::sync::Arc;

use derive_new::new;
use domain::repository::work_parent_packs::WorkParentPacksRepository as _;
use domain::repository::works::DmmWorkRepository as _;
use domain::repository::{
    manager::RepositoryManager, work_like::WorkLikeRepository, work_lnk::WorkLnkRepository,
    works::WorkRepository, RepositoriesExt,
};
use domain::service::work_registration::{
    ImageApply, ImageSource, ImageStrategy, RegisterWorkPath, UniqueWorkKey, WorkInsert,
    WorkRegistrationService,
};
use domain::windows::{shell_link::ShellLink as ShellLinkTrait, WindowsExt};
use domain::works::WorkDetails;
use std::marker::PhantomData;

#[derive(new)]
pub struct WorkUseCase<M, R, W, RS>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
    RS: WorkRegistrationService + Send + Sync + 'static,
{
    manager: Arc<M>,
    windows: Arc<W>,
    registrar: Arc<RS>,
    #[new(default)]
    _marker: PhantomData<R>,
}

impl<M, R, W, RS> WorkUseCase<M, R, W, RS>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
    RS: WorkRegistrationService + Send + Sync + 'static,
{
    pub async fn list_all_details(&self) -> anyhow::Result<Vec<WorkDetails>> {
        self.manager
            .run(|repos| Box::pin(async move { repos.work().list_all_details().await }))
            .await
    }

    pub async fn find_details_by_work_id(
        &self,
        work_id: String,
    ) -> anyhow::Result<Option<WorkDetails>> {
        self.manager
            .run(|repos| {
                Box::pin(async move {
                    repos
                        .work()
                        .find_details_by_work_id(domain::StrId::new(work_id))
                        .await
                })
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

    pub async fn get_parent_dmm_pack_work_id(
        &self,
        work_id: String,
    ) -> anyhow::Result<Option<String>> {
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
        register_path: RegisterWorkPath,
    ) -> anyhow::Result<()> {
        // icon: FromPath/OnlyIfMissing (path がある場合のみ)
        let icon = Some(ImageApply {
            strategy: ImageStrategy::OnlyIfMissing,
            source: ImageSource::FromPath(register_path.clone()),
        });

        // thumbnail: FromUrl/OnlyIfMissing (URL がある場合のみ)
        let thumbnail = if thumbnail_url.is_empty() {
            None
        } else {
            Some(ImageApply {
                strategy: ImageStrategy::OnlyIfMissing,
                source: ImageSource::FromUrl(thumbnail_url),
            })
        };

        let req = domain::service::work_registration::WorkRegistrationRequest {
            keys: vec![UniqueWorkKey::ErogamescapeId(erogamescape_id)],
            insert: WorkInsert {
                title,
                path: Some(register_path),
                egs_info: None,
                icon,
                thumbnail,
                parent_pack_work_id: None,
            },
        };

        let _ = self.registrar.register(vec![req]).await?;
        Ok(())
    }
}
