use chrono::Local;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use domain::{
    repository::{
        manager::RepositoryManager,
        work_download_path::WorkDownloadPathRepository,
        works::{DlsiteWorkRepository, DmmWorkRepository},
        RepositoriesExt,
    },
    scan::{DuplicateResolver, FileSystem, ResolvedWork},
    service::{
        save_path_resolver::SavePathResolver,
        work_linker::{WorkLinkTask, WorkLinker},
    },
    StrId,
};

pub struct DownloadsUseCase<U, R, FS, DR, WL>
where
    U: RepositoryManager<R> + Send + Sync + 'static,
    R: RepositoriesExt + Send + Sync + 'static,
    FS: FileSystem,
    DR: DuplicateResolver,
    WL: WorkLinker,
{
    pub manager: Arc<U>,
    pub resolver: Arc<dyn SavePathResolver>,
    fs: Arc<FS>,
    dedup: Arc<DR>,
    linker: Arc<WL>,
    _marker: std::marker::PhantomData<R>,
}

impl<U, R, FS, DR, WL> DownloadsUseCase<U, R, FS, DR, WL>
where
    U: RepositoryManager<R> + Send + Sync + 'static,
    R: RepositoriesExt + Send + Sync + 'static,
    FS: FileSystem + Send + Sync + 'static,
    DR: DuplicateResolver + Send + Sync + 'static,
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
        dedup: Arc<DR>,
        linker: Arc<WL>,
    ) -> Self {
        Self {
            manager,
            resolver,
            fs,
            dedup,
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

        // 変換: 列挙結果を ResolvedWork に詰める（タイトルはファイル名）
        let mut resolved: Vec<ResolvedWork> = Vec::new();
        for candidate in iter {
            // MEMO: title, egs_id, distance は必要ない(dedupの除外ファイル名のロジックだけを使いたい)ため適当な値を入れる
            resolved.push(ResolvedWork::new(
                candidate,
                "downloaded".to_string(),
                1,
                1.0,
            ));
        }

        // 候補なしなら終了
        if resolved.is_empty() {
            log::warn!(
                "no executable candidates found in {}",
                install_dir.display()
            );
            return Ok(());
        }

        // 重複排除（除外ルールに基づき候補を選別）
        let deduped = self.dedup.resolve(resolved);
        // 全除外なら終了
        if deduped.is_empty() {
            log::warn!(
                "deduplication removed all candidates for {}",
                install_dir.display()
            );
            return Ok(());
        }

        // タスク化: WorkLinkTask に変換
        let tasks: Vec<WorkLinkTask> = deduped
            .into_iter()
            .map(|item| WorkLinkTask {
                work_id: work_id.clone(),
                kind: item.candidate.kind.clone(),
                src: item.candidate.path,
            })
            .collect();

        // 実行: リンク作成を委譲
        self.linker.ensure_links(tasks).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::repository::mock::{TestRepositories, TestRepositoryManager};
    use domain::scan::{CandidateKind, MockDuplicateResolver, MockFileSystem, WorkCandidate};
    use domain::service::save_path_resolver::SavePathResolver;
    use domain::service::work_linker::MockWorkLinker;
    use std::path::PathBuf;
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
    async fn register_installed_work_単一exeでリンク作成() {
        let temp = TempDir::new().unwrap();
        let resolver_impl = Arc::new(TestResolver::new(temp.path().to_path_buf()));
        let resolver: Arc<dyn SavePathResolver> = resolver_impl.clone();
        let manager = Arc::new(TestRepositoryManager::new(TestRepositories::default()));

        let mut fs = MockFileSystem::new();
        fs.expect_walk_dir().returning(|roots, _| {
            assert_eq!(roots.len(), 1);
            let candidate = WorkCandidate::new(roots[0].join("game.exe"), CandidateKind::Exe);
            Ok(Box::new(vec![candidate].into_iter()))
        });
        let fs = Arc::new(fs);

        let mut dedup = MockDuplicateResolver::new();
        dedup.expect_resolve().returning(|items| items);
        let dedup = Arc::new(dedup);

        let mut linker = MockWorkLinker::new();
        linker.expect_ensure_links().times(1).returning(|tasks| {
            assert_eq!(tasks.len(), 1);
            assert!(tasks[0].src.ends_with("game.exe"));
            Box::pin(async { Ok(()) })
        });
        let linker = Arc::new(linker);

        let uc = DownloadsUseCase::new(manager, resolver, fs, dedup, linker);
        let install_dir = temp.path().join("installed");
        std::fs::create_dir_all(&install_dir).unwrap();

        uc.register_installed_work(StrId::new("10".to_string()), &install_dir)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn handle_single_展開後にリンク登録される() {
        let temp = TempDir::new().unwrap();
        let resolver_impl = Arc::new(TestResolver::new(temp.path().to_path_buf()));
        let resolver: Arc<dyn SavePathResolver> = resolver_impl.clone();

        let repos = TestRepositories::default();
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

        let mut dedup = MockDuplicateResolver::new();
        dedup.expect_resolve().returning(|items| items);
        let dedup = Arc::new(dedup);

        let mut linker = MockWorkLinker::new();
        linker.expect_ensure_links().times(1).returning(|tasks| {
            assert_eq!(tasks.len(), 1);
            assert!(tasks[0].src.ends_with("game.exe"));
            Box::pin(async { Ok(()) })
        });
        let linker = Arc::new(linker);

        let uc = DownloadsUseCase::new(manager, resolver, fs, dedup, linker);

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
}
