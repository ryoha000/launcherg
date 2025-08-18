//! ネイティブホスト同期ユースケース: ストア(DMM/DLsite)のゲーム情報をコレクションへ同期する。
//! - 既存マッピングがあればスキップして冪等性を保つ
//! - EGS 情報があれば名称/詳細も upsert し、EGS マップを作成/更新する

use std::sync::Arc;
use derive_new::new;
use crate::domain::repository::collection::CollectionRepository;
use crate::infrastructure::repositoryimpl::repository::RepositoriesExt;
use crate::domain::{thumbnail::ThumbnailService, icon::IconService};

/// 拡張から渡された image_url/thumbnail_url を保存に適したサムネイルURLへ正規化する
/// - DLsite: /resize/images2/.../_img_main_300x300.jpg → /modpub/images2/.../_img_main.jpg
/// - DMM:    ...ps.jpg → ...pl.jpg
fn normalize_thumbnail_url(src_url: &str) -> String {
	let mut url = src_url.to_string();
	if url.contains("img.dlsite.jp") {
		url = url.replace("/resize/images2/", "/modpub/images2/");
		if let Some(last_slash) = url.rfind('/') {
			let (base, file) = url.split_at(last_slash + 1);
			if let Some(main_pos) = file.find("_img_main_") {
				let (prefix, _) = file.split_at(main_pos);
				url = format!("{}{}_img_main.jpg", base, prefix);
			}
		}
	} else if url.contains("pics.dmm.co.jp") {
		if url.ends_with("ps.jpg") {
			url = url.trim_end_matches("ps.jpg").to_string() + "pl.jpg";
		}
	}
	url
}

#[derive(Clone, Debug)]
/// DMM 由来のゲーム同期パラメータ。キーは `(store_id, category, subcategory)`。
/// - `egs` が `Some` の場合、EGS 情報をコレクションへ反映し、EGS マップも作成/更新する。
/// - `egs` が `None` の場合、与えられた `gamename` を用いて要素IDを採番し、マッピングのみ作成する。
pub struct DmmSyncGameParam {
	pub store_id: String,
	pub category: String,
	pub subcategory: String,
	pub gamename: String,
	pub egs: Option<EgsInfo>,
	pub image_url: String,
}

#[derive(Clone, Debug)]
/// DLsite 由来のゲーム同期パラメータ。キーは `(store_id, category)`。
/// 取り扱いは DMM と同様で、`egs` の有無に応じてコレクション側の更新内容が変わる。
pub struct DlsiteSyncGameParam {
	pub store_id: String,
	pub category: String,
	pub gamename: String,
	pub egs: Option<EgsInfo>,
	pub image_url: String,
}

#[derive(Clone, Debug)]
/// ErogameScape(EGS) 由来のメタ情報。
/// コレクション要素の名称・詳細情報に反映され、`erogamescape_id` は EGS マップのキーとなる。
pub struct EgsInfo {
	pub erogamescape_id: i32,
	pub gamename: String,
	pub gamename_ruby: String,
	pub brandname: String,
	pub brandname_ruby: String,
	pub sellday: String,
	pub is_nukige: bool,
}

#[derive(new)]
/// ストア情報をコレクションへ同期するユースケース。
/// 内部で `CollectionRepository` を用いてマッピング作成・EGS 情報反映を行う。
pub struct NativeHostSyncUseCase<R: RepositoriesExt, TS: ThumbnailService, IS: IconService> {
	repositories: Arc<R>,
	thumbnails: Arc<TS>,
	icons: Arc<IS>,
}

impl<R: RepositoriesExt, TS: ThumbnailService, IS: IconService> NativeHostSyncUseCase<R, TS, IS> {
	/// 指定 EGS に対応するコレクション要素を確実に用意する。
	/// - 既存があれば名称・詳細を上書き更新
	/// - なければ新規採番し、EGS マップ・名称・詳細を作成
	/// 戻り値: コレクション要素 ID
	async fn ensure_collection_for_egs(
		&self,
		egs: &EgsInfo,
	) -> anyhow::Result<crate::domain::Id<crate::domain::collection::CollectionElement>> {
		let collection_element_id;
		if let Some(cid) = self
			.repositories
			.collection_repository()
			.get_collection_id_by_erogamescape_id(egs.erogamescape_id)
			.await?
		{
			collection_element_id = cid;
		} else {
			// 新規採番し、EGSマップを作成
			let cid = self
				.repositories
				.collection_repository()
				.allocate_new_collection_element_id(&egs.gamename)
				.await?;
			self.repositories
				.collection_repository()
				.upsert_erogamescape_map(&cid, egs.erogamescape_id)
				.await?;
			collection_element_id = cid;
		}

		// erogamescape 由来の詳細情報を upsert
		let info: crate::domain::collection::NewCollectionElementInfo = crate::domain::collection::NewCollectionElementInfo::new(
			collection_element_id.clone(),
			egs.gamename_ruby.clone(),
			egs.brandname.clone(),
			egs.brandname_ruby.clone(),
			egs.sellday.clone(),
			egs.is_nukige,
		);
		self
			.repositories
			.collection_repository()
			.upsert_collection_element_info(&info)
			.await?;

		Ok(collection_element_id)
	}

	/// EGS 不明用の要素を採番して作成する。
	/// - 与えられた `gamename` をそのまま `collection_elements` に登録する
	/// 戻り値: コレクション要素 ID
	async fn create_collection_without_egs(
		&self,
		gamename: &str,
	) -> anyhow::Result<crate::domain::Id<crate::domain::collection::CollectionElement>> {
		self
			.repositories
			.collection_repository()
			.allocate_new_collection_element_id(gamename)
			.await
	}

	/// DMM のゲーム情報を同期する。
	/// - 既存チェック: `(store_id, category, subcategory)` が存在すればスキップ（冪等）
	/// - `egs: Some` の場合、EGS に紐づく要素を用意・更新した上で DMM マッピングを upsert
	/// - `egs: None` の場合、空要素を採番し DMM マッピングのみ upsert
	/// 戻り値: 新規に作成/更新した件数
	/// エラー: 最初に失敗した地点で早期終了し伝播
	pub async fn sync_dmm_games(
		&self,
		games: Vec<DmmSyncGameParam>,
	) -> anyhow::Result<u32> {
		let mut success: u32 = 0;
		for DmmSyncGameParam { store_id, category, subcategory, gamename, egs, image_url } in games {
			// 既存 (store_id, category, subcategory) がある場合はスキップ
			let exists = self
				.repositories
				.collection_repository()
				.get_collection_id_by_dmm_mapping(&store_id, &category, &subcategory)
				.await?;
			if let Some(_) = exists {
				continue;
			}
			let collection_element_id;
			match egs.as_ref() {
				Some(egs) => {
					collection_element_id = self.ensure_collection_for_egs(egs).await?;
					self.repositories
						.collection_repository()
						.upsert_dmm_mapping(&collection_element_id, &store_id, &category, &subcategory)
						.await?;
				}
				None => {
					collection_element_id = self.create_collection_without_egs(&gamename).await?;
					self.repositories
						.collection_repository()
						.upsert_dmm_mapping(&collection_element_id, &store_id, &category, &subcategory)
						.await?;
				}
			}
			if !image_url.is_empty() {
				// アイコンの保存（URLから、短辺基準で縮小→中央正方形切り抜き）
				let _ = self.icons.save_icon_from_url(&collection_element_id, &image_url).await;
				// サムネイルの保存
				let normalized = normalize_thumbnail_url(&image_url);
				let _ = self.thumbnails.save_thumbnail(&collection_element_id, &normalized).await;
			}
			success += 1;
		}
		Ok(success)
	}

	/// DLsite のゲーム情報を同期する。
	/// - 既存チェック: `(store_id, category)` が存在すればスキップ（冪等）
	/// - `egs: Some` の場合、EGS に紐づく要素を用意・更新した上で DLsite マッピングを upsert
	/// - `egs: None` の場合、空要素を採番し DLsite マッピングのみ upsert
	/// 戻り値: 新規に作成/更新した件数
	/// エラー: 最初に失敗した地点で早期終了し伝播
	pub async fn sync_dlsite_games(
		&self,
		games: Vec<DlsiteSyncGameParam>,
	) -> anyhow::Result<u32> {
		let mut success: u32 = 0;
		for DlsiteSyncGameParam { store_id, category, gamename, egs, image_url } in games {
			// 既存 (store_id, category) がある場合はスキップ
			let exists = self
				.repositories
				.collection_repository()
				.get_collection_id_by_dlsite_mapping(&store_id, &category)
				.await?;
			if let Some(_) = exists {
				continue;
			}
			let collection_element_id;
			match egs.as_ref() {
				Some(egs) => {
					collection_element_id = self.ensure_collection_for_egs(egs).await?;
					self.repositories
						.collection_repository()
						.upsert_dlsite_mapping(&collection_element_id, &store_id, &category)
						.await?;
				}
				None => {
					collection_element_id = self.create_collection_without_egs(&gamename).await?;
					self.repositories
						.collection_repository()
						.upsert_dlsite_mapping(&collection_element_id, &store_id, &category)
						.await?;
				}
			}
			if !image_url.is_empty() {
				// アイコンの保存（URLから、短辺基準で縮小→中央正方形切り抜き）
				let _ = self.icons.save_icon_from_url(&collection_element_id, &image_url).await;
				// サムネイルの保存
				let normalized = normalize_thumbnail_url(&image_url);
				let _ = self.thumbnails.save_thumbnail(&collection_element_id, &normalized).await;
			}
			success += 1;
		}
		Ok(success)
	}
}

// ========== 移行: 旧 `native_host.rs` のユーティリティ群 ==========

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct HostStatusStore {
	last_sync_seconds: Option<i64>,
	total_synced: u32,
	recent_extension_ids: Vec<String>,
}

/// ネイティブホスト用のルートディレクトリ
pub fn host_root_dir() -> String {
	// %APPDATA%\ryoha.moe\launcherg
	let base = dirs::config_dir().unwrap_or(std::env::current_dir().unwrap());
	let path = base.join("ryoha.moe").join("launcherg");
	std::fs::create_dir_all(&path).ok();
	path.to_string_lossy().to_string()
}

fn status_file_path() -> String { format!("{}/native_host_status.json", host_root_dir()) }
fn config_file_path() -> String { format!("{}/native_host_config.json", host_root_dir()) }
pub fn db_file_path() -> String { format!("{}/launcherg_sqlite.db3", host_root_dir()) }

fn load_status_store() -> HostStatusStore {
	let p = status_file_path();
	match std::fs::read_to_string(&p) {
		Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
		Err(_) => HostStatusStore::default(),
	}
}

fn save_status_store(store: HostStatusStore) {
	let p = status_file_path();
	let _ = std::fs::write(p, serde_json::to_string_pretty(&store).unwrap_or("{}".to_string()));
}

/// 拡張機能の設定を保存
pub fn save_config(config: &crate::domain::extension::ExtensionConfig) -> anyhow::Result<()> {
	let p = config_file_path();
	std::fs::write(p, serde_json::to_string_pretty(config).unwrap_or("{}".to_string()))?;
	Ok(())
}

#[derive(Clone, Debug)]
pub struct HostStatusData {
	pub last_sync_seconds: Option<i64>,
	pub total_synced: u32,
	pub connected_extensions: Vec<String>,
}

/// 現在の同期ステータスを取得
pub fn get_status_data() -> HostStatusData {
	let s = load_status_store();
	HostStatusData {
		last_sync_seconds: s.last_sync_seconds,
		total_synced: s.total_synced,
		connected_extensions: s.recent_extension_ids,
	}
}

/// 同期カウンタを更新
pub fn bump_sync_counters(success_add: u32) {
	let mut s = load_status_store();
	s.last_sync_seconds = Some(chrono::Utc::now().timestamp());
	s.total_synced = s.total_synced.saturating_add(success_add);
	save_status_store(s);
}

#[cfg(test)]
mod tests;


