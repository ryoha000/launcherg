use std::sync::Arc;

use derive_new::new;
use domain::repository::{manager::RepositoryManager, RepositoriesExt};
use domain::scan::CandidateKind;
use domain::service::work_linker::{WorkLinkTask, WorkLinker};
use domain::work_link_pending_exe::WorkLinkPendingExeRepository;
use std::marker::PhantomData;

#[derive(new)]
pub struct WorkLinkPendingExeUseCase<M, R, WL>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    WL: WorkLinker + Send + Sync + 'static,
{
    manager: Arc<M>,
    linker: Arc<WL>,
    #[new(default)]
    _marker: PhantomData<R>,
}

impl<M, R, WL> WorkLinkPendingExeUseCase<M, R, WL>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    WL: WorkLinker + Send + Sync + 'static,
{
    /// work_link_pending_exe テーブルからレコードを取得し、.lnk を作成して登録後、テーブルから削除
    pub async fn process_pending_exe_links(&self) -> anyhow::Result<()> {
        let linker = self.linker.clone();
        self.manager
            .run_in_transaction(|repos| {
                let linker = linker.clone();
                Box::pin(async move {
                    // 1) すべての pending exe レコードを取得
                    let pending_list = repos.work_link_pending_exe().list_all().await?;

                    if pending_list.is_empty() {
                        return Ok::<(), anyhow::Error>(());
                    }

                    // 2) WorkLinkTask に変換し、削除対象の id も一緒に保持
                    let mut tasks = Vec::new();
                    let mut delete_ids = Vec::new();
                    for pending in pending_list {
                        delete_ids.push(pending.id);
                        tasks.push(WorkLinkTask {
                            work_id: pending.work_id,
                            kind: CandidateKind::Exe,
                            src: std::path::PathBuf::from(pending.exe_path),
                        });
                    }

                    // 3) WorkLinker を使用して .lnk を作成・登録
                    linker.ensure_links(tasks).await?;

                    // 4) 成功したレコードを削除
                    for id in delete_ids {
                        repos.work_link_pending_exe().delete(id).await?;
                    }

                    Ok::<(), anyhow::Error>(())
                })
            })
            .await
    }
}
