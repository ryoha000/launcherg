use std::path::PathBuf;

pub trait SavePathResolver: Send + Sync {
	fn root_dir(&self) -> String;

	fn icons_dir(&self) -> String { self.join_and_ensure("game-icons") }
	fn thumbnails_dir(&self) -> String { self.join_and_ensure("thumbnails") }
	fn screenshots_dir(&self) -> String { self.join_and_ensure("screenshots") }
	fn memos_dir(&self) -> String { self.join_and_ensure("game-memos") }
	fn play_histories_dir(&self) -> String { self.join_and_ensure("play-histories") }
	fn db_file_path(&self) -> String { PathBuf::from(self.root_dir()).join("launcherg_sqlite.db3").to_string_lossy().to_string() }

	fn icon_png_path(&self, id: i32) -> String { PathBuf::from(self.icons_dir()).join(format!("{}.png", id)).to_string_lossy().to_string() }
	fn thumbnail_png_path(&self, id: i32) -> String { PathBuf::from(self.thumbnails_dir()).join(format!("{}.png", id)).to_string_lossy().to_string() }
	fn play_history_jsonl_path(&self, id: i32) -> String { PathBuf::from(self.play_histories_dir()).join(format!("{}.jsonl", id)).to_string_lossy().to_string() }
	fn memo_image_dir(&self, id: i32) -> String { let p = PathBuf::from(self.memos_dir()).join(format!("{}", id)); std::fs::create_dir_all(&p).ok(); p.to_string_lossy().to_string() }

	// Generate new memo image path like game-memos/{id}/{UUID}.png (directory ensured)
	fn memo_image_new_png_path(&self, id: i32) -> String {
		let dir = PathBuf::from(self.memo_image_dir(id));
		dir.join(format!("{}.png", uuid::Uuid::new_v4().to_string())).to_string_lossy().to_string()
	}

	// Generate screenshot file path with name-timestamp under screenshots dir (directory ensured)
	fn screenshot_png_path_with_name(&self, name: &str) -> String {
		let dir = PathBuf::from(self.screenshots_dir());
		let timestamp = chrono::Local::now().format("%Y-%m-%d-%H-%M-%S");
		dir.join(format!("{}-{}.png", name, timestamp)).to_string_lossy().to_string()
	}

	// Default memo markdown path like game-memos/{id}/untitled.md (directory ensured)
	fn memo_default_md_path(&self, id: i32) -> String {
		let dir = PathBuf::from(self.memo_image_dir(id));
		dir.join("untitled.md").to_string_lossy().to_string()
	}

	fn tmp_download_path_for_queue(&self, queue_id: i32, url: &str) -> String {
		let mut dir = std::env::temp_dir();
		dir.push("launcherg-image-queue");
		std::fs::create_dir_all(&dir).ok();
		let filename = self.filename_from_url(url);
		PathBuf::from(dir).join(format!("{}-{}", queue_id, filename)).to_string_lossy().to_string()
	}

	fn tmp_download_path_for_id(&self, id: i32, url: &str) -> String {
		let mut dir = std::env::temp_dir();
		dir.push("launcherg-images");
		std::fs::create_dir_all(&dir).ok();
		let filename = self.filename_from_url(url);
		PathBuf::from(dir).join(format!("{}-{}", id, filename)).to_string_lossy().to_string()
	}

	fn filename_from_url(&self, url: &str) -> String {
		url::Url::parse(url)
			.ok()
			.and_then(|u| u.path_segments().and_then(|s| s.last()).map(|s| s.to_string()))
			.unwrap_or_else(|| "image".to_string())
	}

	// helper
	fn join_and_ensure(&self, sub: &str) -> String {
		let p = PathBuf::from(self.root_dir()).join(sub);
		std::fs::create_dir_all(&p).ok();
		p.to_string_lossy().to_string()
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


