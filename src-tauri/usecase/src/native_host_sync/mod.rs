//! ネイティブホスト同期ユースケース: ストア(DMM/DLsite)のゲーム情報をコレクションへ同期する。
//! - 既存マッピングがあればスキップして冪等性を保つ
//! - EGS 情報があれば名称/詳細も upsert し、EGS マップを作成/更新する

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use domain::repository::work_omit::WorkOmitRepository;
use domain::repository::works::{DmmWorkRepository, DlsiteWorkRepository};
use derive_new::new;
use domain::repository::{collection::CollectionRepository, RepositoriesExt};
use domain::repository::works::WorkRepository;
use domain::repository::work_parent_packs::WorkParentPacksRepository;
use domain::save_image_queue::{ImageSrcType, ImagePreprocess};
use domain::repository::save_image_queue::ImageSaveQueueRepository;
use domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};

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

// 旧 build_icon_dst_path / build_thumbnail_resized_dst_path は SavePathResolver に移管

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
    pub parent_pack_work_id: Option<i32>,
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
/// DMM の (store_id, category, subcategory) を表すキー。
struct DmmKey {
	store_id: String,
	category: String,
	subcategory: String,
}

#[derive(Clone, Debug)]
/// sync 前に一括取得しておくスナップショット
struct DmmBatchSnapshot {
	/// 入力キーに対する DMM Work の有無
	work_id_by_key: HashMap<DmmKey, Option<i32>>,
	/// 既存マッピング済みキーの一覧
	mapped_keys: HashMap<DmmKey, domain::Id<domain::collection::CollectionElement>>,
	/// omit 済みの work_id 集合
	omitted_work_ids: HashSet<i32>,
	/// 事前取得した EGS ID -> CollectionElementId マップ
	egs_id_to_collection_id: HashMap<i32, domain::Id<domain::collection::CollectionElement>>,
}

#[derive(Clone, Debug)]
/// EGS→Collection のメモ化など、計画/実行間で共有するキャッシュ
struct Caches {
	pub egs_id_to_collection_id: HashMap<i32, domain::Id<domain::collection::CollectionElement>>,
}

impl Default for Caches {
	fn default() -> Self { Self { egs_id_to_collection_id: HashMap::new() } }
}

#[derive(Clone, Debug)]
struct SyncApply {
	key: DmmKey,
	work_id_opt: Option<i32>,
	gamename: String,
	image_url: String,
	parent_pack_work_id: Option<i32>,
	egs: Option<EgsInfo>,
}

#[derive(Clone, Debug)]
enum PlanDecision {
	SkipExists,
	SkipOmitted,
	Apply(SyncApply),
}

impl DmmKey {
	fn from_param(p: &DmmSyncGameParam) -> DmmKey {
		DmmKey { store_id: p.store_id.clone(), category: p.category.clone(), subcategory: p.subcategory.clone() }
	}
}

impl<R: RepositoriesExt> NativeHostSyncUseCase<R> {
    /// 1件分の計画を決定（純粋ロジック + omit は IO）。
    async fn decide_for_game(&self, snapshot: &DmmBatchSnapshot, param: DmmSyncGameParam) -> anyhow::Result<PlanDecision> {
        let key = DmmKey::from_param(&param);

        // 既存マッピングならスキップ
        if snapshot.mapped_keys.contains_key(&key) {
            return Ok(PlanDecision::SkipExists);
        }

		let work_id = snapshot.work_id_by_key.get(&key).cloned().unwrap_or(None);
        // omit が付いていればスキップ（work_id がある場合のみ）
        if let Some(work_id) = work_id {
            if snapshot.omitted_work_ids.contains(&work_id) {
                return Ok(PlanDecision::SkipOmitted);
            }
        }

        Ok(PlanDecision::Apply(SyncApply {
            key,
            work_id_opt: work_id,
            gamename: param.gamename,
            image_url: param.image_url,
            parent_pack_work_id: param.parent_pack_work_id,
            egs: param.egs,
        }))
    }
}

#[derive(new)]
/// ストア情報をコレクションへ同期するユースケース。
/// 内部で `CollectionRepository` を用いてマッピング作成・EGS 情報反映を行う。
pub struct NativeHostSyncUseCase<R: RepositoriesExt> {
	repositories: Arc<tokio::sync::Mutex<R>>,
	resolver: Arc<dyn SavePathResolver>,
}

impl<R: RepositoriesExt> NativeHostSyncUseCase<R> {
	/// DMM 同期用のバッチスナップショットを構築する
	async fn build_dmm_batch_snapshot(
		&self,
		games: &[DmmSyncGameParam],
	) -> anyhow::Result<DmmBatchSnapshot> {
		// 入力からキー一覧を作成
		let keys: Vec<(String, String, String)> = games
			.iter()
			.map(|g| (g.store_id.clone(), g.category.clone(), g.subcategory.clone()))
			.collect();

		// Work の一括取得
		let mut work_id_by_key: HashMap<DmmKey, Option<i32>> = HashMap::new();
		{
			let mut repos = self.repositories.lock().await;
			let works = repos.dmm_work().find_by_store_keys(&keys).await?;
			for w in works.into_iter() {
				work_id_by_key.insert(
					DmmKey { store_id: w.store_id.clone(), category: w.category.clone(), subcategory: w.subcategory.clone() },
					Some(w.id.value),
				);
			}
			// 入力に対して必ずキーを用意（未取得は None）
			for (sid, cat, sub) in keys.iter() {
				work_id_by_key.entry(DmmKey { store_id: sid.clone(), category: cat.clone(), subcategory: sub.clone() }).or_insert(None);
			}
		}

		// 既存マッピングの一括取得（work_id 直引き）
		let mut mapped_keys: HashMap<DmmKey, domain::Id<domain::collection::CollectionElement>> = HashMap::new();
		{
			// HashMap の列挙順に依存しないよう、入力順 `keys` に基づいて work_ids を構築する
			let work_ids: Vec<i32> = keys
				.iter()
				.filter_map(|(sid, cat, sub)| {
					work_id_by_key
						.get(&DmmKey { store_id: sid.clone(), category: cat.clone(), subcategory: sub.clone() })
						.and_then(|v| *v)
				})
				.collect();
			let mut repos = self.repositories.lock().await;
			let existing = repos.collection().get_collection_ids_by_work_ids(&work_ids).await?;
			// work_id -> CE を key へ戻す
			let mut keys_by_work: HashMap<i32, Vec<DmmKey>> = HashMap::new();
			for (k, v) in work_id_by_key.iter() {
				if let Some(wid) = v { keys_by_work.entry(*wid).or_default().push(k.clone()); }
			}
			for (wid, ce) in existing.into_iter() {
				if let Some(keys_for_w) = keys_by_work.get(&wid) {
					for k in keys_for_w.iter() { mapped_keys.insert(k.clone(), ce.clone()); }
				}
			}
		}

		// omit 一括取得
		let omitted_work_ids: HashSet<i32> = {
			let mut repos = self.repositories.lock().await;
			repos
				.work_omit()
				.list()
				.await?
				.into_iter()
				.map(|o| o.work_id.value)
				.collect()
		};

		// EGS 一括取得
		let mut egs_id_to_collection_id: HashMap<i32, domain::Id<domain::collection::CollectionElement>> = HashMap::new();
		let egs_ids: Vec<i32> = games
			.iter()
			.filter_map(|g| g.egs.as_ref().map(|e| e.erogamescape_id))
			.collect();
		if !egs_ids.is_empty() {
			let mut repos = self.repositories.lock().await;
			let rows = repos.collection().get_collection_ids_by_erogamescape_ids(&egs_ids).await?;
			for (egs_id, ceid) in rows.into_iter() { egs_id_to_collection_id.insert(egs_id, ceid); }
		}

		Ok(DmmBatchSnapshot { work_id_by_key, mapped_keys, omitted_work_ids, egs_id_to_collection_id })
	}

	/// DMM 作品の画像をキュー投入する（アイコン/サムネ/別名パス）。
	async fn enqueue_images_for_dmm(
		&self,
		collection_element_id: &domain::Id<domain::collection::CollectionElement>,
		category: &str,
		subcategory: &str,
		store_id: &str,
		image_url: &str,
	) -> anyhow::Result<()> {
		if image_url.is_empty() { return Ok(()); }

		let icon_dst = self.resolver.icon_png_path(collection_element_id.value);
		{
			let mut repos = self.repositories.lock().await;
			let _ = repos.image_queue()
				.enqueue(image_url, ImageSrcType::Url, &icon_dst, ImagePreprocess::ResizeAndCropSquare256)
				.await;
		}

		let normalized = normalize_thumbnail_url(image_url);
		let thumb_dst = self.resolver.thumbnail_png_path(collection_element_id.value);
		{
			let mut repos = self.repositories.lock().await;
			let _ = repos.image_queue()
				.enqueue(&normalized, ImageSrcType::Url, &thumb_dst, ImagePreprocess::ResizeForWidth400)
				.await;
		}

		let alias = self.resolver.thumbnail_alias_dmm_png_path(category, subcategory, store_id);
		{
			let mut repos = self.repositories.lock().await;
			let _ = repos.image_queue()
				.enqueue(&normalized, ImageSrcType::Url, &alias, ImagePreprocess::ResizeForWidth400)
				.await;
		}

		Ok(())
	}

	/// 計画にもとづき副作用を実行（要素用意→マッピング→親子リンク→画像投入）
	async fn execute_apply(&self, apply: SyncApply, caches: &mut Caches) -> anyhow::Result<()> {
		let SyncApply { key, work_id_opt, gamename, image_url, parent_pack_work_id, egs } = apply;

		// Collection Element を用意
		let collection_element_id = match egs.as_ref() {
			Some(egs_info) => {
				if let Some(cid) = caches.egs_id_to_collection_id.get(&egs_info.erogamescape_id) {
					cid.clone()
				} else {
					let cid = self.ensure_collection_for_egs(egs_info).await?;
					caches.egs_id_to_collection_id.insert(egs_info.erogamescape_id, cid.clone());
					cid
				}
			}
			None => {
				self.create_collection_without_egs(&gamename).await?
			}
		};

		// Work があればマッピング/親子リンク
		if let Some(work_id) = work_id_opt {
			let mut repos = self.repositories.lock().await;
			repos
				.collection()
				.upsert_work_mapping(&collection_element_id, work_id)
				.await?;
			if let Some(pid) = parent_pack_work_id {
				let _ = repos.work_parent_packs().add(domain::Id::new(work_id), domain::Id::new(pid)).await;
			}
		}

		// 画像投入（失敗は握りつぶして継続する実装方針を踏襲）
		let _ = self.enqueue_images_for_dmm(&collection_element_id, &key.category, &key.subcategory, &key.store_id, &image_url).await;

		Ok(())
	}

	/// 指定 EGS に対応するコレクション要素を確実に用意する。
	/// - 既存があれば名称・詳細を上書き更新
	/// - なければ新規採番し、EGS マップ・名称・詳細を作成
	/// 戻り値: コレクション要素 ID
	async fn ensure_collection_for_egs(
		&self,
		egs: &EgsInfo,
	) -> anyhow::Result<domain::Id<domain::collection::CollectionElement>> {
		let collection_element_id;
		if let Some(cid) = {
			let mut repos = self.repositories.lock().await;
			repos
				.collection()
				.get_collection_id_by_erogamescape_id(egs.erogamescape_id)
				.await?
		} {
			collection_element_id = cid;
		} else {
			// 新規採番し、EGSマップを作成
			let cid = {
				let mut repos = self.repositories.lock().await;
				repos
					.collection()
					.allocate_new_collection_element_id(&egs.gamename)
					.await?
			};
			{
				let mut repos = self.repositories.lock().await;
				repos
					.collection()
					.upsert_erogamescape_map(&cid, egs.erogamescape_id)
					.await?;
			}
			collection_element_id = cid;
		}

		// erogamescape 由来の詳細情報を upsert
		let info: domain::collection::NewCollectionElementInfo = domain::collection::NewCollectionElementInfo::new(
			collection_element_id.clone(),
			egs.gamename_ruby.clone(),
			egs.brandname.clone(),
			egs.brandname_ruby.clone(),
			egs.sellday.clone(),
			egs.is_nukige,
		);
		{
			let mut repos = self.repositories.lock().await;
			repos
				.collection()
				.upsert_collection_element_info(&info)
				.await?;
		}

		Ok(collection_element_id)
	}

	/// EGS 不明用の要素を採番して作成する。
	/// - 与えられた `gamename` をそのまま `collection_elements` に登録する
	/// 戻り値: コレクション要素 ID
	async fn create_collection_without_egs(
		&self,
		gamename: &str,
	) -> anyhow::Result<domain::Id<domain::collection::CollectionElement>> {
		let mut repos = self.repositories.lock().await;
		repos
			.collection()
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
		let snapshot = self.build_dmm_batch_snapshot(&games).await?;
		let mut caches = Caches { egs_id_to_collection_id: snapshot.egs_id_to_collection_id.clone() };

		for param in games.into_iter() {
			match self.decide_for_game(&snapshot, param).await? {
				PlanDecision::SkipExists => {}
				PlanDecision::SkipOmitted => {}
				PlanDecision::Apply(apply) => {
					self.execute_apply(apply, &mut caches).await?;
					success += 1;
				}
			}
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
		// omit は都度 exists 判定（work_id ベース）
		for DlsiteSyncGameParam { store_id, category, gamename, egs, image_url } in games {
			if let Some(work) = {
				let mut repos = self.repositories.lock().await;
				repos.dlsite_work().find_by_store_key(&store_id, &category).await?
			} {
				if {
					let mut repos = self.repositories.lock().await;
					repos.work_omit().exists(domain::Id::new(work.id.value)).await?
				} { continue; }
			}
			// 既存 (store_id, category) がある場合はスキップ
			let exists = {
				let mut repos = self.repositories.lock().await;
				repos
					.collection()
					.get_collection_id_by_dlsite_mapping(&store_id, &category)
					.await?
			};
			if let Some(_) = exists {
				continue;
			}
			let collection_element_id;
			match egs.as_ref() {
				Some(egs) => {
					collection_element_id = self.ensure_collection_for_egs(egs).await?;
					if let Some(work) = {
						let mut repos = self.repositories.lock().await;
						repos.dlsite_work().find_by_store_key(&store_id, &category).await?
					} {
						let mut repos = self.repositories.lock().await;
						repos
							.collection()
							.upsert_work_mapping(&collection_element_id, work.id.value)
							.await?;
					}
				}
				None => {
					collection_element_id = self.create_collection_without_egs(&gamename).await?;
					if let Some(work) = {
						let mut repos = self.repositories.lock().await;
						repos.dlsite_work().find_by_store_key(&store_id, &category).await?
					} {
						let mut repos = self.repositories.lock().await;
						repos
							.collection()
							.upsert_work_mapping(&collection_element_id, work.id.value)
							.await?;
					}
				}
			}
			if !image_url.is_empty() {
				let icon_dst = self.resolver.icon_png_path(collection_element_id.value);
				{
					let mut repos = self.repositories.lock().await;
					let _ = repos.image_queue()
						.enqueue(&image_url, ImageSrcType::Url, &icon_dst, ImagePreprocess::ResizeAndCropSquare256)
						.await;
				}
				let normalized = normalize_thumbnail_url(&image_url);
				let thumb_dst = self.resolver.thumbnail_png_path(collection_element_id.value);
				{
					let mut repos = self.repositories.lock().await;
					let _ = repos.image_queue()
						.enqueue(&normalized, ImageSrcType::Url, &thumb_dst, ImagePreprocess::ResizeForWidth400)
						.await;
				}
				// DLsite 作品の別名パスでも保存
				let alias = self.resolver.thumbnail_alias_dlsite_png_path(&category, &store_id);
				{
					let mut repos = self.repositories.lock().await;
					let _ = repos.image_queue()
						.enqueue(&normalized, ImageSrcType::Url, &alias, ImagePreprocess::ResizeForWidth400)
						.await;
				}
			}
			success += 1;
		}
		Ok(success)
	}

	/// DMM の omit が付与された作品の一覧を返す（DMM情報必須）。
	pub async fn list_dmm_omit_works(&self) -> anyhow::Result<Vec<DmmOmitItem>> {
		let all = {
			let mut repos = self.repositories.lock().await;
			repos.work().list_all_details().await?
		};
		let mut out: Vec<DmmOmitItem> = Vec::new();
		for w in all.into_iter() {
			if w.is_dmm_omitted {
				if let Some(dmm) = w.dmm {
					out.push(DmmOmitItem {
						work_id: w.work.id.value,
						store_id: dmm.store_id,
						category: dmm.category,
						subcategory: dmm.subcategory,
					});
				}
			}
		}
		Ok(out)
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
	// SavePathResolver へ統一
	DirsSavePathResolver::default().root_dir()
}

pub fn db_file_path() -> String { DirsSavePathResolver::default().db_file_path() }

#[derive(Clone, Debug)]
pub struct HostStatusData {
	pub last_sync_seconds: Option<i64>,
	pub total_synced: u32,
	pub connected_extensions: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct DmmOmitItem {
	pub work_id: i32,
	pub store_id: String,
	pub category: String,
	pub subcategory: String,
}

#[cfg(test)]
mod tests;


