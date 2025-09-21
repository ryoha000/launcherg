use std::collections::HashSet;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

use domain::repository::manager::RepositoryManager;
use domain::repository::work_lnk::NewWorkLnk;
use domain::repository::{work_lnk::WorkLnkRepository as _, RepositoriesExt};
use domain::scan::CandidateKind;
use domain::service::save_path_resolver::SavePathResolver;
use domain::service::work_linker::{WorkLinkTask, WorkLinker};
use domain::windows::shell_link::{CreateShortcutRequest, ShellLink as _};
use domain::windows::WindowsExt;

pub struct WorkLinkerImpl<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt,
{
    manager: Arc<M>,
    resolver: Arc<dyn SavePathResolver>,
    windows: Arc<W>,
    _marker: PhantomData<R>,
}

impl<M, R, W> WorkLinkerImpl<M, R, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt,
{
    pub fn new(manager: Arc<M>, resolver: Arc<dyn SavePathResolver>, windows: Arc<W>) -> Self {
        Self {
            manager,
            resolver,
            windows,
            _marker: PhantomData,
        }
    }
}

#[derive(Clone)]
struct PreparedTask {
    work_id: domain::Id<domain::works::Work>,
    kind: CandidateKind,
    src: String,
    dst: String,
}

impl<M, R, W> WorkLinker for WorkLinkerImpl<M, R, W>
where
    M: RepositoryManager<R> + Send + Sync + 'static,
    R: RepositoriesExt + Send + Sync + 'static,
    W: WindowsExt + Send + Sync + 'static,
{
    async fn ensure_links(&self, tasks: Vec<WorkLinkTask>) -> Result<()> {
        // 入力が空なら何もしない
        if tasks.is_empty() {
            return Ok(());
        }

        // 事前整形: 対象種別のみ抽出し、work_id ごとに一意化してリンク先を決定
        let mut seen: HashSet<i32> = HashSet::new();
        let mut prepared: Vec<PreparedTask> = Vec::new();
        for task in tasks.into_iter() {
            if !matches!(task.kind, CandidateKind::Exe | CandidateKind::Shortcut) {
                continue;
            }
            if seen.insert(task.work_id.value) {
                let dst = self.resolver.lnk_new_path(task.work_id.value);
                prepared.push(PreparedTask {
                    work_id: task.work_id,
                    kind: task.kind,
                    src: task.src.to_string_lossy().to_string(),
                    dst,
                });
            }
        }
        // 対象が無ければ終了
        if prepared.is_empty() {
            return Ok(());
        }

        // リンク作成: Exe は作成リクエストを蓄積、Shortcut はコピーして登録候補に追加
        let mut exe_reqs: Vec<CreateShortcutRequest> = Vec::new();
        let mut to_insert: Vec<(domain::Id<domain::works::Work>, String)> = Vec::new();
        for task in prepared.iter() {
            if let Some(parent) = Path::new(&task.dst).parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match task.kind {
                CandidateKind::Shortcut => {
                    if std::fs::copy(&task.src, &task.dst).is_ok() {
                        to_insert.push((task.work_id.clone(), task.dst.clone()));
                    }
                }
                CandidateKind::Exe => {
                    exe_reqs.push(CreateShortcutRequest {
                        target_path: task.src.clone(),
                        dest_lnk_path: task.dst.clone(),
                        working_dir: None,
                        arguments: None,
                        icon_path: None,
                    });
                }
                _ => {}
            }
        }

        // Exe の .lnk を一括作成し、作成結果を登録候補に追加
        if !exe_reqs.is_empty() {
            self.windows.shell_link().create_bulk(exe_reqs)?;
            for task in prepared
                .iter()
                .filter(|t| matches!(t.kind, CandidateKind::Exe))
            {
                to_insert.push((task.work_id.clone(), task.dst.clone()));
            }
        }

        // 作成できたものが無ければ DB 登録は不要
        if to_insert.is_empty() {
            return Ok(());
        }

        // DB 登録: 作成済みリンクのパスを work_lnk に保存（トランザクション）
        self.manager
            .run_in_transaction(|repos| {
                let records = to_insert.clone();
                Box::pin(async move {
                    let mut repo = repos.work_lnk();
                    for (work_id, path) in records.into_iter() {
                        repo.insert(&NewWorkLnk {
                            work_id,
                            lnk_path: path,
                        })
                        .await?;
                    }
                    Ok::<_, anyhow::Error>(())
                })
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::repository::mock::{TestRepositories, TestRepositoryManager};
    use domain::service::save_path_resolver::SavePathResolver;
    use domain::windows::{
        process::MockProcessWindows, proctail::MockProcTail,
        proctail_manager::MockProcTailManagerTrait, shell_link::MockShellLink, WindowsExt,
    };
    use tempfile::TempDir;

    #[derive(Clone)]
    struct TestResolver {
        root: String,
    }

    impl TestResolver {
        fn new(root: String) -> Self {
            Self { root }
        }
    }

    impl SavePathResolver for TestResolver {
        fn root_dir(&self) -> String {
            self.root.clone()
        }
    }

    struct TestWindows {
        process: MockProcessWindows,
        proctail: MockProcTail,
        proctail_manager: MockProcTailManagerTrait,
        shell_link: MockShellLink,
    }

    impl TestWindows {
        fn new(shell_link: MockShellLink) -> Self {
            Self {
                process: MockProcessWindows::new(),
                proctail: MockProcTail::new(),
                proctail_manager: MockProcTailManagerTrait::new(),
                shell_link,
            }
        }
    }

    impl WindowsExt for TestWindows {
        type ProcessWindows = MockProcessWindows;
        type ProcTail = MockProcTail;
        type ProcTailManager = MockProcTailManagerTrait;
        type ShellLink = MockShellLink;

        fn process(&self) -> &Self::ProcessWindows {
            &self.process
        }
        fn proctail(&self) -> &Self::ProcTail {
            &self.proctail
        }
        fn proctail_manager(&self) -> &Self::ProcTailManager {
            &self.proctail_manager
        }
        fn shell_link(&self) -> &Self::ShellLink {
            &self.shell_link
        }
    }

    #[tokio::test]
    async fn ensure_links_creates_shortcut_for_exe() {
        let tmp = TempDir::new().unwrap();
        let exe_path = tmp.path().join("game.exe");
        std::fs::write(&exe_path, b"dummy").unwrap();

        let repos = TestRepositories::default();
        {
            let mut work_lnk = repos.work_lnk.lock().await;
            work_lnk
                .expect_list_by_work_id()
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
            work_lnk
                .expect_insert()
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(domain::Id::new(1)) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos.clone()));

        let resolver = Arc::new(TestResolver::new(tmp.path().to_string_lossy().to_string()));

        let mut shell = MockShellLink::new();
        shell
            .expect_create_bulk()
            .withf(|reqs| reqs.len() == 1 && reqs[0].target_path.ends_with("game.exe"))
            .returning(|_| Ok(()));
        let windows = Arc::new(TestWindows::new(shell));

        let linker: WorkLinkerImpl<_, _, _> = WorkLinkerImpl::new(manager, resolver, windows);
        let task = WorkLinkTask {
            work_id: domain::Id::new(10),
            kind: CandidateKind::Exe,
            src: exe_path.clone(),
        };

        let res = linker.ensure_links(vec![task]).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn ensure_links_skips_existing_records() {
        let tmp = TempDir::new().unwrap();
        let lnk_dir = tmp.path().join("lnk-source");
        std::fs::create_dir_all(&lnk_dir).unwrap();
        let shortcut_src = lnk_dir.join("game.lnk");
        std::fs::write(&shortcut_src, b"shortcut").unwrap();

        let repos = TestRepositories::default();
        {
            let mut work_lnk = repos.work_lnk.lock().await;
            work_lnk.expect_list_by_work_id().returning(|_| {
                Box::pin(async {
                    Ok::<_, anyhow::Error>(vec![domain::repository::work_lnk::WorkLnk {
                        id: domain::Id::new(1),
                        work_id: domain::Id::new(20),
                        lnk_path: "existing".into(),
                    }])
                })
            });
        }
        let manager = Arc::new(TestRepositoryManager::new(repos.clone()));
        let resolver = Arc::new(TestResolver::new(tmp.path().to_string_lossy().to_string()));

        let shell = MockShellLink::new();
        let windows = Arc::new(TestWindows::new(shell));

        let linker: WorkLinkerImpl<_, _, _> = WorkLinkerImpl::new(manager, resolver, windows);
        let task = WorkLinkTask {
            work_id: domain::Id::new(20),
            kind: CandidateKind::Shortcut,
            src: shortcut_src,
        };

        let res = linker.ensure_links(vec![task]).await;
        assert!(res.is_ok());
    }
}
