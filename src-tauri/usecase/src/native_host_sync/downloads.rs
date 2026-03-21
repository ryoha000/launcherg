use chrono::Local;
use std::cmp::Reverse;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use domain::{
    game_matcher::config::{INSTALL_EXCLUDE_DIR_NAMES, INSTALL_HELPER_NAMES, INSTALL_PRIORITY_NAMES},
    repository::{
        manager::RepositoryManager,
        work_download_path::WorkDownloadPathRepository,
        works::{DlsiteWorkRepository, DmmWorkRepository, WorkRepository},
        RepositoriesExt,
    },
    scan::FileSystem,
    service::{
        save_path_resolver::SavePathResolver,
        work_linker::{WorkLinkTask, WorkLinker},
    },
    StrId,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct InstallCandidateRank {
    kind_score: i32,
    exact_name_score: i32,
    helper_score: i32,
    stem_len: Reverse<usize>,
    path: Reverse<String>,
}

pub struct DownloadsUseCase<U, R, FS, WL>
where
    U: RepositoryManager<R> + Send + Sync + 'static,
    R: RepositoriesExt + Send + Sync + 'static,
    FS: FileSystem,
    WL: WorkLinker,
{
    pub manager: Arc<U>,
    pub resolver: Arc<dyn SavePathResolver>,
    fs: Arc<FS>,
    linker: Arc<WL>,
    _marker: std::marker::PhantomData<R>,
}

impl<U, R, FS, WL> DownloadsUseCase<U, R, FS, WL>
where
    U: RepositoryManager<R> + Send + Sync + 'static,
    R: RepositoriesExt + Send + Sync + 'static,
    FS: FileSystem + Send + Sync + 'static,
    WL: WorkLinker + Send + Sync + 'static,
{
    /// DownloadsUseCase を生成する。
    ///
    /// - manager: リポジトリ操作を仲介するマネージャ
    /// - resolver: ダウンロード先ディレクトリを解決するサービス
    pub fn new(
        manager: Arc<U>,
        resolver: Arc<dyn SavePathResolver>,
        fs: Arc<FS>,
        linker: Arc<WL>,
    ) -> Self {
        Self {
            manager,
            resolver,
            fs,
            linker,
            _marker: std::marker::PhantomData,
        }
    }
    /// DMM の `store_id` から、対応する作品 (`Work`) の ID を検索して返す。
    /// 見つからない場合はエラーを返す。
    pub async fn resolve_dmm_work_id(
        &self,
        store_id: &str,
    ) -> anyhow::Result<StrId<domain::works::Work>> {
        let maybe = self
            .manager
            .run(|repos| {
                let sid = store_id.to_string();
                Box::pin(async move {
                    Ok::<_, anyhow::Error>(
                        repos
                            .dmm_work()
                            .find_by_store_id(&sid)
                            .await?
                            .map(|w| w.work_id),
                    )
                })
            })
            .await?;
        if let Some(id) = maybe {
            Ok(id)
        } else {
            anyhow::bail!(format!("dmm work not found: store_id={}", store_id))
        }
    }

    /// DLsite の `store_id` から対応する作品 ID を検索して返す。
    /// 見つからない場合はエラーを返す。
    pub async fn resolve_dlsite_work_id(
        &self,
        store_id: &str,
    ) -> anyhow::Result<StrId<domain::works::Work>> {
        let sid = store_id.to_string();
        let maybe = self
            .manager
            .run(move |repos| {
                let sid2 = sid.clone();
                Box::pin(async move {
                    Ok::<_, anyhow::Error>(
                        repos
                            .dlsite_work()
                            .find_by_store_id(&sid2)
                            .await?
                            .map(|w| w.work_id),
                    )
                })
            })
            .await?;
        if let Some(id) = maybe {
            Ok(id)
        } else {
            anyhow::bail!(format!("dlsite work not found: store_id={}", store_id))
        }
    }

    /// ダウンロードした作品の保存先パスを記録する。
    /// `work_id` が `None` の場合は何もせず成功として返す。
    pub async fn save_download_path(
        &self,
        work_id: StrId<domain::works::Work>,
        path: &str,
    ) -> anyhow::Result<()> {
        let p = path.to_string();
        self.manager
            .run(|repos| {
                let p2 = p.clone();
                Box::pin(async move { repos.work_download_path().add(work_id, &p2).await })
            })
            .await?;
        Ok(())
    }

    /// ダウンロード済みゲームのルートディレクトリの絶対パスを返す。
    pub fn downloaded_games_dir(&self) -> String {
        self.resolver.downloaded_games_dir()
    }

    /// 単一ファイル（例: `.zip`）またはディレクトリを取り込み、
    /// 必要に応じて展開または移動して保存先を記録する。
    ///
    /// - `.zip` の場合: 同名フォルダを作成して展開（Windows では PowerShell の Expand-Archive を使用）
    /// - ディレクトリの場合: ルート直下に移動
    /// - それ以外のファイルはエラー
    pub async fn handle_single(
        &self,
        filename: &str,
        work_id: StrId<domain::works::Work>,
    ) -> anyhow::Result<PathBuf> {
        let dst_root = PathBuf::from(self.downloaded_games_dir());
        let src = Path::new(filename);
        let name = src
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("download");
        let lower = name.to_lowercase();
        // 決定: 保存先は `${work_id}_yyyymmddhhmmss`（存在時は `_2` 以降）
        let dst_dir = self
            .make_unique_work_subdir(&dst_root, work_id.clone())
            .await?;
        if lower.ends_with(".zip") {
            std::fs::create_dir_all(&dst_dir).ok();
            std::process::Command::new("powershell")
                .args([
                    "-Command",
                    &format!(
                        "Expand-Archive -Force -Path \"{}\" -DestinationPath \"{}\"",
                        src.display(),
                        dst_dir.display()
                    ),
                ])
                .status()?;
        } else if src.is_dir() {
            std::fs::create_dir_all(&dst_root).ok();
            std::fs::rename(src, &dst_dir)?;
        } else {
            anyhow::bail!("unsupported file: {}", name);
        }
        self.save_download_path(work_id.clone(), &dst_dir.to_string_lossy())
            .await?;
        self.register_installed_work(work_id, &dst_dir).await?;
        Ok(dst_dir)
    }

    /// 分割アーカイブ等を想定し、項目リストから自己解凍 `exe` を探して実行し展開する。
    /// 実行時は `-d<展開先>` と `-s` を付与（Windows 限定）。
    /// 実行可能ファイルが見つからない場合はエラーを返す。
    pub async fn handle_split(
        &self,
        items: &[String],
        work_id: StrId<domain::works::Work>,
    ) -> anyhow::Result<PathBuf> {
        let dst_root = PathBuf::from(self.downloaded_games_dir());
        std::fs::create_dir_all(&dst_root).ok();
        // 保存先は `${work_id}_yyyymmddhhmmss`（存在時は `_2` 以降）
        let dst_dir = self
            .make_unique_work_subdir(&dst_root, work_id.clone())
            .await?;
        std::fs::create_dir_all(&dst_dir).ok();
        if let Some(exe) = items.iter().find(|p| p.to_lowercase().ends_with(".exe")) {
            std::process::Command::new(exe)
                .args([&format!("-d{}", dst_dir.display()), "-s"]) // -d に空白不可
                .current_dir(Path::new(exe).parent().unwrap_or(Path::new(".")))
                .status()?;
            self.save_download_path(work_id.clone(), &dst_dir.to_string_lossy())
                .await?;
            self.register_installed_work(work_id, &dst_dir).await?;
            Ok(dst_dir)
        } else {
            anyhow::bail!("no executable found")
        }
    }

    /// `${work_id}_yyyymmddhhmmss` 形式のサブディレクトリを作成し、既存時は `_2`, `_3` ... を付けて一意化して返す。
    async fn make_unique_work_subdir(
        &self,
        dst_root: &Path,
        work_id: StrId<domain::works::Work>,
    ) -> anyhow::Result<PathBuf> {
        let wid = work_id.value;
        let now = Local::now();
        let ts = now.format("%Y%m%d%H%M%S").to_string();
        let base_name = format!("{}_{}", wid, ts);
        let mut candidate = dst_root.join(&base_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
        let mut suffix = 2;
        loop {
            let name = format!("{}_{}", base_name, suffix);
            candidate = dst_root.join(&name);
            if !candidate.exists() {
                return Ok(candidate);
            }
            suffix += 1;
        }
    }

    pub async fn register_installed_work(
        &self,
        work_id: StrId<domain::works::Work>,
        install_dir: &Path,
    ) -> anyhow::Result<()> {
        // 候補列挙: インストールディレクトリ以下を走査
        let roots = vec![install_dir.to_path_buf()];
        let iter = match self.fs.walk_dir(&roots, None) {
            Ok(it) => it,
            Err(e) => {
                log::warn!(
                    "failed to enumerate candidates: {} ({})",
                    install_dir.display(),
                    e
                );
                return Ok(());
            }
        };

        // 変換: 列挙結果から、主となる実行ファイル/ショートカットを 1 本選ぶ
        let mut best_task: Option<WorkLinkTask> = None;
        let mut best_rank: Option<InstallCandidateRank> = None;
        for candidate in iter {
            if !matches!(
                &candidate.kind,
                domain::scan::CandidateKind::Exe | domain::scan::CandidateKind::Shortcut
            ) {
                continue;
            }

            let task = WorkLinkTask {
                work_id: work_id.clone(),
                kind: candidate.kind.clone(),
                src: candidate.path,
            };
            if Self::is_excluded_install_candidate(&task.src) {
                continue;
            }
            let rank = Self::rank_install_candidate(install_dir, &task.kind, &task.src);
            if best_rank.as_ref().map_or(true, |current| rank > *current) {
                best_rank = Some(rank);
                best_task = Some(task);
            }
        }

        // 候補なしなら終了
        let Some(task) = best_task else {
            log::warn!(
                "no executable candidates found in {}",
                install_dir.display()
            );
            return Ok(());
        };

        self.record_install_metadata(work_id.clone(), &task.src).await;

        // 実行: リンク作成を委譲
        self.linker.ensure_links(vec![task]).await?;
        Ok(())
    }

    fn rank_install_candidate(
        install_dir: &Path,
        kind: &domain::scan::CandidateKind,
        path: &Path,
    ) -> InstallCandidateRank {
        let kind_score = match kind {
            domain::scan::CandidateKind::Exe => 2,
            domain::scan::CandidateKind::Shortcut => 1,
            _ => 0,
        };

        let path_string = path.to_string_lossy().to_string();
        let file_name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_lowercase();
        let stem = path
            .file_stem()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_lowercase();
        let parent = path
            .parent()
            .and_then(|v| v.file_name())
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_lowercase();
        let install_dir_name = install_dir
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_lowercase();
        let file_name_is_priority = INSTALL_PRIORITY_NAMES
            .iter()
            .any(|name| file_name == *name);
        let file_name_is_helper = INSTALL_HELPER_NAMES
            .iter()
            .any(|name| file_name == *name);

        // 主exeは「インストール先ディレクトリ名」またはその直下の親フォルダ名と
        // 名前が近いことが多い。helper や補助ツールは名前がずれやすいので、
        // ここではディレクトリ名/親フォルダ名と一致する候補を優先する。
        // さらに、既知の主 exe 名は完全一致で強く優先し、既知の helper 名は
        // 完全一致で強く後回しにする。
        let exact_name_score = if file_name_is_priority {
            2
        } else if !install_dir_name.is_empty()
            && (stem.contains(&install_dir_name) || install_dir_name.contains(&stem))
        {
            1
        } else if !parent.is_empty() && (stem.contains(&parent) || parent.contains(&stem)) {
            1
        } else {
            0
        };

        let helper_penalty = if file_name_is_helper {
            3
        } else {
            0
        };

        InstallCandidateRank {
            kind_score,
            exact_name_score,
            helper_score: -helper_penalty,
            stem_len: Reverse(stem.len()),
            path: Reverse(path_string),
        }
    }

    fn is_excluded_install_candidate(path: &Path) -> bool {
        path.ancestors()
            .filter_map(|ancestor| ancestor.file_name())
            .filter_map(|name| name.to_str())
            .map(|name| name.to_lowercase())
            .any(|name| INSTALL_EXCLUDE_DIR_NAMES.iter().any(|exclude| name == *exclude))
    }

    async fn record_install_metadata(&self, work_id: StrId<domain::works::Work>, original_path: &Path) {
        if let Ok(meta) = std::fs::metadata(original_path) {
            let created = meta.created().ok();
            let modified = meta.modified().ok();
            if let Some(best_st) = match (created, modified) {
                (Some(c), Some(m)) => Some(if m > c { m } else { c }),
                (Some(c), None) => Some(c),
                (None, Some(m)) => Some(m),
                _ => None,
            } {
                let best_dt_local = chrono::DateTime::<chrono::Utc>::from(best_st)
                    .with_timezone(&chrono::Local);
                let original_path_string = original_path.to_string_lossy().to_string();
                let work_id_value = work_id.value.clone();
                if let Err(e) = self
                    .manager
                    .run(|repos| {
                        let original_path = original_path_string.clone();
                        let work_id = work_id.clone();
                        let best_dt_local = best_dt_local;
                        Box::pin(async move {
                            repos
                                .work()
                                .update_install_by_work_id(work_id, best_dt_local, original_path)
                                .await
                        })
                    })
                    .await
                {
                    log::warn!(
                        "Failed to update install_at/original_path for work_id={}: {}",
                        work_id_value,
                        e
                    );
                }
            }
        } else {
            log::warn!(
                "Failed to get metadata for path: {}",
                original_path.display()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::repository::mock::{TestRepositories, TestRepositoryManager};
    use domain::scan::{CandidateKind, MockFileSystem, WorkCandidate};
    use domain::service::save_path_resolver::SavePathResolver;
    use domain::service::work_linker::MockWorkLinker;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    #[derive(Clone)]
    struct TestResolver {
        root: PathBuf,
    }

    impl TestResolver {
        fn new(root: PathBuf) -> Self {
            Self { root }
        }
    }

    impl SavePathResolver for TestResolver {
        fn root_dir(&self) -> String {
            self.root.to_string_lossy().to_string()
        }
    }

    #[tokio::test]
    async fn handle_single_展開後にリンク登録される() {
        let temp = TempDir::new().unwrap();
        let resolver_impl = Arc::new(TestResolver::new(temp.path().to_path_buf()));
        let resolver: Arc<dyn SavePathResolver> = resolver_impl.clone();

        let repos = TestRepositories::default();
        {
            let mut work_repo = repos.work.lock().await;
            work_repo.expect_update_install_by_work_id().returning(|_, _, original_path| {
                assert!(original_path.ends_with("game.exe"));
                Box::pin(async { Ok::<_, anyhow::Error>(()) })
            });
        }
        {
            let mut work_download_path = repos.work_download_path.lock().await;
            work_download_path
                .expect_add()
                .returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));

        let mut fs = MockFileSystem::new();
        fs.expect_walk_dir().returning(|roots, _| {
            assert_eq!(roots.len(), 1);
            let candidate = WorkCandidate::new(roots[0].join("game.exe"), CandidateKind::Exe);
            Ok(Box::new(vec![candidate].into_iter()))
        });
        let fs = Arc::new(fs);
        let mut linker = MockWorkLinker::new();
        linker.expect_ensure_links().times(1).returning(|tasks| {
            assert_eq!(tasks.len(), 1);
            assert!(tasks[0].src.ends_with("game.exe"));
            Box::pin(async { Ok(()) })
        });
        let linker = Arc::new(linker);

        let uc = DownloadsUseCase::new(manager, resolver, fs, linker);

        let source_dir = TempDir::new().unwrap();
        let exe_path = source_dir.path().join("game.exe");
        std::fs::write(&exe_path, b"dummy").unwrap();

        let result_path = uc
            .handle_single(
                source_dir.path().to_string_lossy().as_ref(),
                StrId::new("42".to_string()),
            )
            .await
            .unwrap();
        assert!(result_path.starts_with(temp.path()));
        assert!(result_path.join("game.exe").exists());
    }

    #[tokio::test]
    async fn register_installed_work_候補選定テーブルテスト() {
        struct Case<'a> {
            name: &'a str,
            candidates: Vec<(&'a str, CandidateKind)>,
            expected_file_name: Option<&'a str>,
        }

        let cases = vec![
            Case {
                name: "単一 exe はそのまま採用する",
                candidates: vec![("game.exe", CandidateKind::Exe)],
                expected_file_name: Some("game.exe"),
            },
            Case {
                name: "helper より main exe を優先する",
                candidates: vec![
                    ("notification_helper.exe", CandidateKind::Exe),
                    ("Game.exe", CandidateKind::Exe),
                ],
                expected_file_name: Some("Game.exe"),
            },
            Case {
                name: "主 exe 名を完全一致で優先する",
                candidates: vec![
                    ("notification_helper.exe", CandidateKind::Exe),
                    ("Game.exe", CandidateKind::Exe),
                ],
                expected_file_name: Some("Game.exe"),
            },
            Case {
                name: "Start 系の候補群では Start.exe を採用する",
                candidates: vec![
                    ("Start.exe", CandidateKind::Exe),
                    ("StartData/Config.exe", CandidateKind::Exe),
                    ("StartData/StartMenu.exe", CandidateKind::Exe),
                    ("StartData/TraceLog.exe", CandidateKind::Exe),
                    ("StartData/GameData/SiglusEngine.exe", CandidateKind::Exe),
                ],
                expected_file_name: Some("Start.exe"),
            },
            Case {
                name: "UnityCrashHandler64.exe は helper として後回しにする",
                candidates: vec![
                    ("CYANBRAIN.exe", CandidateKind::Exe),
                    ("UnityCrashHandler64.exe", CandidateKind::Exe),
                ],
                expected_file_name: Some("CYANBRAIN.exe"),
            },
            Case {
                name: "エンジン設定.exe は helper として後回しにする",
                candidates: vec![
                    ("lol.exe", CandidateKind::Exe),
                    ("エンジン設定.exe", CandidateKind::Exe),
                ],
                expected_file_name: Some("lol.exe"),
            },
            Case {
                name: "startup.exe は主 exe として優先する",
                candidates: vec![
                    ("startup.exe", CandidateKind::Exe),
                    ("SupportTools.exe", CandidateKind::Exe),
                    ("ハロー・レディ！/hellolady_srp.exe", CandidateKind::Exe),
                    ("ハロー・レディ！/hellolady.exe", CandidateKind::Exe),
                    ("ハロー・レディ！/ハロー・レディ！.exe", CandidateKind::Exe),
                ],
                expected_file_name: Some("startup.exe"),
            },
            Case {
                name: "SETUP 配下の候補は除外する",
                candidates: vec![
                    ("Setup.exe", CandidateKind::Exe),
                    ("SETUP/もも☆プラ.eXe", CandidateKind::Exe),
                    ("Game.exe", CandidateKind::Exe),
                ],
                expected_file_name: Some("Game.exe"),
            },
            Case {
                name: "setup.exe は主候補として優先する",
                candidates: vec![
                    ("setup.exe", CandidateKind::Exe),
                    ("files/uninst.exe", CandidateKind::Exe),
                    ("files/zombie.exe", CandidateKind::Exe),
                ],
                expected_file_name: Some("setup.exe"),
            },
            Case {
                name: "SupportTools.exe は helper として後回しにする",
                candidates: vec![
                    ("realsister_.exe", CandidateKind::Exe),
                    ("SupportTools.exe", CandidateKind::Exe),
                ],
                expected_file_name: Some("realsister_.exe"),
            },
        ];

        for case in cases {
            let case_name = case.name;
            let candidates = case.candidates.clone();
            let expected_file_name = case.expected_file_name;
            let temp = TempDir::new().unwrap();
            let resolver_impl = Arc::new(TestResolver::new(temp.path().to_path_buf()));
            let resolver: Arc<dyn SavePathResolver> = resolver_impl.clone();
            let repos = TestRepositories::default();

            if let Some(expected_file_name) = expected_file_name {
                let mut work_repo = repos.work.lock().await;
                work_repo
                    .expect_update_install_by_work_id()
                    .returning(move |_, _, original_path| {
                        let actual = Path::new(&original_path)
                            .file_name()
                            .and_then(|v| v.to_str())
                            .unwrap_or_default();
                        assert_eq!(actual, expected_file_name, "case: {}", case_name);
                        Box::pin(async { Ok::<_, anyhow::Error>(()) })
                    });
            }
            let manager = Arc::new(TestRepositoryManager::new(repos));

            let mut fs = MockFileSystem::new();
            fs.expect_walk_dir().returning(move |roots, _| {
                assert_eq!(roots.len(), 1, "case: {}", case_name);
                let items = candidates
                    .iter()
                    .map(|(rel, kind)| WorkCandidate::new(roots[0].join(rel), kind.clone()))
                    .collect::<Vec<_>>();
                Ok(Box::new(items.into_iter()))
            });
            let fs = Arc::new(fs);

            let mut linker = MockWorkLinker::new();
            if let Some(expected_file_name) = expected_file_name {
                linker.expect_ensure_links().times(1).returning(move |tasks| {
                    assert_eq!(tasks.len(), 1, "case: {}", case_name);
                    let actual = tasks[0]
                        .src
                        .file_name()
                        .and_then(|v| v.to_str())
                        .unwrap_or_default();
                    assert_eq!(actual, expected_file_name, "case: {}", case_name);
                    Box::pin(async { Ok(()) })
                });
            }
            let linker = Arc::new(linker);

            let uc = DownloadsUseCase::new(manager, resolver, fs, linker);
            let install_dir = temp.path().join("installed");
            std::fs::create_dir_all(&install_dir).unwrap();

            uc.register_installed_work(StrId::new("10".to_string()), &install_dir)
                .await
                .unwrap();
        }
    }
}
