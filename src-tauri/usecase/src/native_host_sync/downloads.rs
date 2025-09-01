use std::path::{Path, PathBuf};
use std::sync::Arc;
use domain::{Id, repository::{RepositoriesExt, manager::RepositoryManager, works::{DmmWorkRepository, DlsiteWorkRepository}, work_download_path::WorkDownloadPathRepository}, service::save_path_resolver::SavePathResolver};

pub struct DownloadsUseCase<U, R>
where U: RepositoryManager<R> + Send + Sync + 'static, R: RepositoriesExt + Send + Sync + 'static {
    pub manager: Arc<U>,
    pub resolver: Arc<dyn SavePathResolver>,
    _marker: std::marker::PhantomData<R>,
}

impl<U, R> DownloadsUseCase<U, R>
where U: RepositoryManager<R> + Send + Sync + 'static, R: RepositoriesExt + Send + Sync + 'static {
    /// DownloadsUseCase を生成する。
    ///
    /// - manager: リポジトリ操作を仲介するマネージャ
    /// - resolver: ダウンロード先ディレクトリを解決するサービス
    pub fn new(manager: Arc<U>, resolver: Arc<dyn SavePathResolver>) -> Self {
        Self { manager, resolver, _marker: std::marker::PhantomData }
    }

    /// DMM の `store_id`/`category`/`subcategory` から、対応する作品 (`Work`) の ID を検索して返す。
    /// 見つからない場合は `None` を返す。
    pub async fn resolve_dmm_work_id(&self, store_id: &str, category: &str, subcategory: &str) -> anyhow::Result<Option<Id<domain::works::Work>>> {
        self.manager.run(|repos| {
            let sid = store_id.to_string();
            let cat = category.to_string();
            let sub = subcategory.to_string();
            Box::pin(async move { Ok::<_, anyhow::Error>(repos.dmm_work().find_by_store_key(&sid, &cat, &sub).await?.map(|w| w.work_id)) })
        }).await
    }

    /// DLsite の `store_id` と `category` から対応する作品 ID を検索して返す。
    /// `category` が空または未指定の場合は、代表的なカテゴリ候補（"pro"/"maniax"）を順に探索する。
    pub async fn resolve_dlsite_work_id(&self, store_id: &str, category: Option<&str>) -> anyhow::Result<Option<Id<domain::works::Work>>> {
        let try_once = |sid: String, cat: String| {
            self.manager.run(move |repos| {
                let sid2 = sid.clone();
                let cat2 = cat.clone();
                Box::pin(async move { Ok::<_, anyhow::Error>(repos.dlsite_work().find_by_store_key(&sid2, &cat2).await?.map(|w| w.work_id)) })
            })
        };

        if let Some(cat) = category {
            if !cat.is_empty() {
                return try_once(store_id.to_string(), cat.to_string()).await;
            }
        }

        // カテゴリ未指定時のフォールバック探索
        if let Some(id) = try_once(store_id.to_string(), "pro".to_string()).await? { return Ok(Some(id)); }
        if let Some(id) = try_once(store_id.to_string(), "maniax".to_string()).await? { return Ok(Some(id)); }
        Ok(None)
    }

    /// ダウンロードした作品の保存先パスを記録する。
    /// `work_id` が `None` の場合は何もせず成功として返す。
    pub async fn save_download_path(&self, work_id: Option<Id<domain::works::Work>>, path: &str) -> anyhow::Result<()> {
        if let Some(wid) = work_id {
            let p = path.to_string();
            self.manager.run(|repos| {
                let p2 = p.clone();
                Box::pin(async move { repos.work_download_path().add(wid, &p2).await })
            }).await?;
        }
        Ok(())
    }

    /// ダウンロード済みゲームのルートディレクトリの絶対パスを返す。
    pub fn downloaded_games_dir(&self) -> String { self.resolver.downloaded_games_dir() }

    /// 単一ファイル（例: `.zip`）またはディレクトリを取り込み、
    /// 必要に応じて展開または移動して保存先を記録する。
    ///
    /// - `.zip` の場合: 同名フォルダを作成して展開（Windows では PowerShell の Expand-Archive を使用）
    /// - ディレクトリの場合: ルート直下に移動
    /// - それ以外のファイルはエラー
    pub async fn handle_single(&self, filename: &str, work_id: Option<Id<domain::works::Work>>) -> anyhow::Result<()> {
        let dst_root = PathBuf::from(self.downloaded_games_dir());
        let src = Path::new(filename);
        let name = src.file_name().and_then(|s| s.to_str()).unwrap_or("download");
        let lower = name.to_lowercase();
        if lower.ends_with(".zip") {
            let stem = name.trim_end_matches(".zip");
            let dst_dir = dst_root.join(stem);
            std::fs::create_dir_all(&dst_dir).ok();
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("powershell").args([
                    "-Command",
                    &format!("Expand-Archive -Force -Path \"{}\" -DestinationPath \"{}\"", src.display(), dst_dir.display()),
                ]).status();
            }
            self.save_download_path(work_id, &dst_dir.to_string_lossy()).await?;
        } else if src.is_dir() {
            let dst_dir = dst_root.join(name);
            std::fs::create_dir_all(&dst_root).ok();
            let _ = std::fs::rename(src, &dst_dir);
            self.save_download_path(work_id, &dst_dir.to_string_lossy()).await?;
        } else {
            anyhow::bail!("unsupported file: {}", name);
        }
        Ok(())
    }

    /// 分割アーカイブ等を想定し、項目リストから自己解凍 `exe` を探して実行し展開する。
    /// 実行時は `-d<展開先>` と `-s` を付与（Windows 限定）。
    /// 実行可能ファイルが見つからない場合はエラーを返す。
    pub async fn handle_split(&self, items: &[String], work_id: Option<Id<domain::works::Work>>) -> anyhow::Result<()> {
        let dst_root = PathBuf::from(self.downloaded_games_dir());
        std::fs::create_dir_all(&dst_root).ok();
        if let Some(exe) = items.iter().find(|p| p.to_lowercase().ends_with(".exe")) {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new(exe)
                    .args([&format!("-d{}", dst_root.display()), "-s"]) // -d に空白不可
                    .current_dir(Path::new(exe).parent().unwrap_or(Path::new(".")))
                    .status();
            }
            self.save_download_path(work_id, &dst_root.to_string_lossy()).await?;
            Ok(())
        } else {
            anyhow::bail!("no executable found")
        }
    }
}


