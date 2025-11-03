use std::path::PathBuf;

use crate::{StrId, works::Work};

#[trait_variant::make(Send + Sync)]
#[mockall::automock]
pub trait SavePathResolver {
    fn root_dir(&self) -> String;

    fn icons_dir(&self) -> String {
        self.join_and_ensure("game-icons")
    }
    fn thumbnails_dir(&self) -> String {
        self.join_and_ensure("thumbnails")
    }
    fn screenshots_dir(&self) -> String {
        self.join_and_ensure("screenshots")
    }
    fn lnks_dir(&self) -> String {
        self.join_and_ensure("lnks")
    }
    fn memos_dir(&self) -> String {
        self.join_and_ensure("game-memos")
    }
    fn play_histories_dir(&self) -> String {
        self.join_and_ensure("play-histories")
    }
    fn db_file_path(&self) -> String {
        PathBuf::from(self.root_dir())
            .join("launcherg_sqlite.db3")
            .to_string_lossy()
            .to_string()
    }

    fn icon_png_path(&self, id: &str) -> String {
        PathBuf::from(self.icons_dir())
            .join(format!("{}.png", id))
            .to_string_lossy()
            .to_string()
    }
    fn thumbnail_png_path(&self, id: &str) -> String {
        PathBuf::from(self.thumbnails_dir())
            .join(format!("{}.png", id))
            .to_string_lossy()
            .to_string()
    }
    fn play_history_jsonl_path(&self, work_id: StrId<Work>) -> String {
        PathBuf::from(self.play_histories_dir())
            .join(format!("{}.jsonl", work_id.value))
            .to_string_lossy()
            .to_string()
    }
    fn memo_image_dir(&self, id: &str) -> String {
        let p = PathBuf::from(self.memos_dir()).join(id);
        std::fs::create_dir_all(&p).ok();
        p.to_string_lossy().to_string()
    }

    // Generate new memo image path like game-memos/{id}/{UUID}.png (directory ensured)
    fn memo_image_new_png_path(&self, id: &str) -> String {
        let dir = PathBuf::from(self.memo_image_dir(id));
        dir.join(format!("{}.png", uuid::Uuid::new_v4().to_string()))
            .to_string_lossy()
            .to_string()
    }

    fn lnk_new_path(&self, work_id: &str) -> String {
        let dir = PathBuf::from(self.lnks_dir());
        dir.join(format!(
            "{}-{}.lnk",
            work_id,
            uuid::Uuid::new_v4().to_string()
        ))
        .to_string_lossy()
        .to_string()
    }

    // Generate screenshot file path with name-timestamp under screenshots dir (directory ensured)
    fn screenshot_png_path_with_name(&self, name: &str) -> String {
        let dir = PathBuf::from(self.screenshots_dir());
        let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S");
        dir.join(format!("{}-{}.png", name, timestamp))
            .to_string_lossy()
            .to_string()
    }

    // Default memo markdown path like game-memos/{id}/untitled.md (directory ensured)
    fn memo_default_md_path(&self, id: &str) -> String {
        let dir = PathBuf::from(self.memo_image_dir(id));
        dir.join("untitled.md").to_string_lossy().to_string()
    }

    // 新方針: 一意な一時ファイルパスを生成（UUIDベース）。
    // 拡張子あり/なしの2種を提供する。
    fn tmp_unique_path_with_ext(&self, extension: &str) -> String {
        let mut dir = std::env::temp_dir();
        dir.push("launcherg-temp");
        std::fs::create_dir_all(&dir).ok();
        let name = format!("{}.{}", uuid::Uuid::new_v4().to_string(), extension);
        PathBuf::from(dir).join(name).to_string_lossy().to_string()
    }

    fn tmp_unique_path(&self) -> String {
        let mut dir = std::env::temp_dir();
        dir.push("launcherg-temp");
        std::fs::create_dir_all(&dir).ok();
        PathBuf::from(dir)
            .join(uuid::Uuid::new_v4().to_string())
            .to_string_lossy()
            .to_string()
    }

    // 互換: 旧APIは新APIで代替（将来削除予定）
    fn tmp_download_path_for_queue(&self, _queue_id: i32, url: &str) -> String {
        let ext = PathBuf::from(self.filename_from_url(url))
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_else(|| "bin".to_string());
        self.tmp_unique_path_with_ext(&ext)
    }

    fn tmp_download_path_for_id(&self, _id: i32, url: &str) -> String {
        self.tmp_download_path_for_queue(0, url)
    }

    fn tmp_ensure_path_for_queue(&self, _queue_id: i32, _filepath: &str) -> String {
        self.tmp_unique_path_with_ext("png")
    }

    fn filename_from_url(&self, url: &str) -> String {
        url::Url::parse(url)
            .ok()
            .and_then(|u| {
                u.path_segments()
                    .and_then(|s| s.last())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "image".to_string())
    }

    // helper
    fn join_and_ensure(&self, sub: &str) -> String {
        let p = PathBuf::from(self.root_dir()).join(sub);
        std::fs::create_dir_all(&p).ok();
        p.to_string_lossy().to_string()
    }

    // downloaded games root directory
    fn downloaded_games_dir(&self) -> String {
        self.join_and_ensure("downloaded_games")
    }
}

#[derive(Clone, Default)]
pub struct DirsSavePathResolver;

impl SavePathResolver for DirsSavePathResolver {
    fn root_dir(&self) -> String {
        let base = dirs::config_dir().unwrap_or(std::env::current_dir().unwrap());
        let path = base.join("ryoha.moe").join("launcherg");
        std::fs::create_dir_all(&path).ok();
        path.to_string_lossy().to_string()
    }
}
