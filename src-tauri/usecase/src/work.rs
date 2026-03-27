use std::path::Path;
use std::sync::Arc;

use derive_new::new;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteLaunchReasonCode {
    Launched,
    DesktopNotConnected,
    Timeout,
    WorkNotFound,
    ShortcutNotFound,
    TargetNotInstalled,
    LaunchFailed,
    Unknown,
}

impl RemoteLaunchReasonCode {
    pub fn message(self) -> &'static str {
        match self {
            Self::Launched => "デスクトップアプリでゲームを起動しました。",
            Self::DesktopNotConnected => {
                "デスクトップアプリが Remote Launch に接続していません。Launcherg を起動した状態で再試行してください。"
            }
            Self::Timeout => "デスクトップアプリから起動結果を受信できませんでした。しばらくしてから再試行してください。",
            Self::WorkNotFound => {
                "この作品は現在の PC に同期されていません。ライブラリを再同期してください。"
            }
            Self::ShortcutNotFound => {
                "この作品には起動用ショートカットが登録されていません。PC 側で作品登録状態を確認してください。"
            }
            Self::TargetNotInstalled => {
                "起動先のファイルが見つかりませんでした。ゲームが未インストールか、インストール先が移動された可能性があります。"
            }
            Self::LaunchFailed => {
                "ゲームの起動に失敗しました。PC 側で直接起動できるか確認してください。"
            }
            Self::Unknown => "ゲームを起動できませんでした。しばらくしてから再試行してください。",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteLaunchAck {
    pub ok: bool,
    pub reason_code: RemoteLaunchReasonCode,
    pub message: String,
    pub detail: Option<String>,
}

impl RemoteLaunchAck {
    fn success(detail: Option<String>) -> Self {
        Self {
            ok: true,
            reason_code: RemoteLaunchReasonCode::Launched,
            message: RemoteLaunchReasonCode::Launched.message().to_string(),
            detail,
        }
    }

    fn failure(reason_code: RemoteLaunchReasonCode, detail: Option<String>) -> Self {
        Self {
            ok: false,
            reason_code,
            message: reason_code.message().to_string(),
            detail,
        }
    }
}

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

    pub async fn launch_remote_work(&self, work_id: String) -> RemoteLaunchAck {
        match self.find_details_by_work_id(work_id.clone()).await {
            Ok(Some(_)) => {}
            Ok(None) => {
                return RemoteLaunchAck::failure(RemoteLaunchReasonCode::WorkNotFound, None)
            }
            Err(err) => {
                return RemoteLaunchAck::failure(
                    RemoteLaunchReasonCode::Unknown,
                    Some(err.to_string()),
                )
            }
        }

        let links = match self.list_work_lnks(work_id).await {
            Ok(links) => links,
            Err(err) => {
                return RemoteLaunchAck::failure(
                    RemoteLaunchReasonCode::Unknown,
                    Some(err.to_string()),
                )
            }
        };

        if links.is_empty() {
            return RemoteLaunchAck::failure(RemoteLaunchReasonCode::ShortcutNotFound, None);
        }

        let mut target_not_installed_detail: Option<String> = None;
        let mut launch_failed_detail: Option<String> = None;

        for (lnk_id, lnk_path) in links {
            if !Path::new(&lnk_path).exists() {
                continue;
            }

            let metadatas = match self
                .windows
                .shell_link()
                .get_lnk_metadatas(vec![lnk_path.clone()])
            {
                Ok(metadatas) => metadatas,
                Err(err) => {
                    launch_failed_detail = Some(err.to_string());
                    continue;
                }
            };

            let Some(metadata) = metadatas.get(&lnk_path) else {
                launch_failed_detail = Some(format!("shortcut metadata missing for {}", lnk_path));
                continue;
            };

            if metadata.path.trim().is_empty() || !Path::new(&metadata.path).exists() {
                target_not_installed_detail = Some(format!(
                    "shortcut target path does not exist: {}",
                    metadata.path
                ));
                continue;
            }

            match self.launch_work(false, lnk_id).await {
                Ok(pid) => {
                    let detail = pid.map(|process_id| format!("process id: {process_id}"));
                    return RemoteLaunchAck::success(detail);
                }
                Err(err) => {
                    launch_failed_detail = Some(err.to_string());
                }
            }
        }

        if let Some(detail) = launch_failed_detail {
            return RemoteLaunchAck::failure(RemoteLaunchReasonCode::LaunchFailed, Some(detail));
        }

        if let Some(detail) = target_not_installed_detail {
            return RemoteLaunchAck::failure(
                RemoteLaunchReasonCode::TargetNotInstalled,
                Some(detail),
            );
        }

        RemoteLaunchAck::failure(RemoteLaunchReasonCode::ShortcutNotFound, None)
    }

    pub async fn get_parent_dmm_pack_key(
        &self,
        work_id: String,
    ) -> anyhow::Result<Option<domain::work_parent_pack::ParentPackKey>> {
        let wid = domain::StrId::new(work_id);
        let parent = self
            .manager
            .run(|repos| {
                Box::pin(async move { repos.work_parent_packs().find_parent_key(wid).await })
            })
            .await?;
        Ok(parent)
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
                parent_pack_dmm_key: None,
            },
        };

        let _ = self.registrar.register(vec![req]).await?;
        Ok(())
    }
}
