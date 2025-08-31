//! ネイティブホスト同期ユースケース: ストア(DMM/DLsite)のゲーム情報をコレクションへ同期する。
//! - 既存マッピングがあればスキップして冪等性を保つ
//! - EGS 情報があれば名称/詳細も upsert し、EGS マップを作成/更新する

use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use domain::repository::work_omit::WorkOmitRepository;
use domain::repository::works::{DmmWorkRepository, DlsiteWorkRepository};
use derive_new::new;
use domain::repository::{collection::CollectionRepository, RepositoriesExt, manager::RepositoryManager, work_parent_packs::WorkParentPacksRepository};
use std::marker::PhantomData;
use domain::repository::works::WorkRepository;
use domain::save_image_queue::{ImageSrcType, ImagePreprocess};
use domain::repository::save_image_queue::ImageSaveQueueRepository;
use domain::service::save_path_resolver::{SavePathResolver};

mod dlsite;
mod dmm;

/// 拡張から渡された image_url/thumbnail_url を保存に適したサムネイルURLへ正規化する
/// - DLsite: /resize/images2/.../_img_main_300x300.jpg → /modpub/images2/.../_img_main.jpg
/// - DMM:    ...ps.jpg → ...pl.jpg
pub(crate) fn normalize_thumbnail_url(src_url: &str) -> String {
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

// DMM 向けの計画決定ロジックは dmm.rs へ移動

#[derive(new)]
/// ストア情報をコレクションへ同期するユースケース。
/// 内部で `CollectionRepository` を用いてマッピング作成・EGS 情報反映を行う。
pub struct NativeHostSyncUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
	manager: Arc<M>,
	resolver: Arc<dyn SavePathResolver>,
    #[new(default)] _marker: PhantomData<R>,
}

impl<M, R> NativeHostSyncUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
	/// DMM 同期用のバッチスナップショットを構築する
	// DMM 同期用のバッチスナップショットは dmm.rs に移動

	// DMM 作品の画像キュー投入は dmm.rs に移動

	/// 作品画像をキュー投入する（トランザクション内で repos を直接使用）
	// DMM 作品の画像キュー投入（repos）は dmm.rs に移動

	// DMM の適用実行は dmm.rs に移動

	/// 計画にもとづき副作用を実行（トランザクション内のリポジトリを使用）
	// DMM の適用実行（repos）は dmm.rs に移動

	/// 指定 EGS に対応するコレクション要素を確実に用意する。
	/// - 既存があれば名称・詳細を上書き更新
	/// - なければ新規採番し、EGS マップ・名称・詳細を作成
	/// 戻り値: コレクション要素 ID
	async fn ensure_collection_for_egs(
		&self,
		egs: &EgsInfo,
	) -> anyhow::Result<domain::Id<domain::collection::CollectionElement>> {
		let collection_element_id;
		if let Some(cid) = self.manager.run(|repos| {
			let id = egs.erogamescape_id;
			Box::pin(async move {
				let mut repo = repos.collection();
				repo.get_collection_id_by_erogamescape_id(id).await
			})
		}).await? {
			collection_element_id = cid;
		} else {
			// 新規採番し、EGSマップを作成
			let cid = self.manager.run(|repos| {
				let name = egs.gamename.clone();
				Box::pin(async move {
					let mut repo = repos.collection();
					repo.allocate_new_collection_element_id(&name).await
				})
			}).await?;
			self.manager.run(|repos| {
				let cid = cid.clone();
				let id = egs.erogamescape_id;
				Box::pin(async move {
					let mut repo = repos.collection();
					repo.upsert_erogamescape_map(&cid, id).await
				})
			}).await?;
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
		self.manager.run(|repos| {
			let info = info.clone();
			Box::pin(async move {
				let mut repo = repos.collection();
				repo.upsert_collection_element_info(&info).await
			})
		}).await?;

		Ok(collection_element_id)
	}

	/// 指定 EGS に対応するコレクション要素を確実に用意する（トランザクション内で repos を直接使用）
	pub(crate) async fn ensure_collection_for_egs_with_repos<Rx: RepositoriesExt + Send + Sync + 'static>(
		repos: &Rx,
		egs: &EgsInfo,
	) -> anyhow::Result<domain::Id<domain::collection::CollectionElement>> {
		let collection_element_id;
		if let Some(cid) = {
			let mut repo = repos.collection();
			repo.get_collection_id_by_erogamescape_id(egs.erogamescape_id).await?
		} {
			collection_element_id = cid;
		} else {
			let cid = {
				let mut repo = repos.collection();
				repo.allocate_new_collection_element_id(&egs.gamename).await?
			};
			{
				let mut repo = repos.collection();
				repo.upsert_erogamescape_map(&cid, egs.erogamescape_id).await?;
			}
			collection_element_id = cid;
		}

		let info: domain::collection::NewCollectionElementInfo = domain::collection::NewCollectionElementInfo::new(
			collection_element_id.clone(),
			egs.gamename_ruby.clone(),
			egs.brandname.clone(),
			egs.brandname_ruby.clone(),
			egs.sellday.clone(),
			egs.is_nukige,
		);
		{
			let mut repo = repos.collection();
			repo.upsert_collection_element_info(&info).await?;
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
		self.manager.run(|repos| {
			let name = gamename.to_string();
			Box::pin(async move {
				let mut repo = repos.collection();
				repo.allocate_new_collection_element_id(&name).await
			})
		}).await
	}

	/// EGS 不明用の要素を採番して作成（トランザクション内で repos を直接使用）
	pub(crate) async fn create_collection_without_egs_with_repos<Rx: RepositoriesExt + Send + Sync + 'static>(
		repos: &Rx,
		gamename: &str,
	) -> anyhow::Result<domain::Id<domain::collection::CollectionElement>> {
		let mut repo = repos.collection();
		repo.allocate_new_collection_element_id(gamename).await
	}

	// DMM の同期は dmm.rs に移動

	/// DLsite のゲーム情報を同期する。
	/// - 既存チェック: `(store_id, category)` が存在すればスキップ（冪等）
	/// - `egs: Some` の場合、EGS に紐づく要素を用意・更新した上で DLsite マッピングを upsert
	/// - `egs: None` の場合、空要素を採番し DLsite マッピングのみ upsert
	/// 戻り値: 新規に作成/更新した件数
	/// エラー: 最初に失敗した地点で早期終了し伝播
	// 実装は dlsite.rs へ分割

	/// DMM の omit が付与された作品の一覧を返す（DMM情報必須）。
	pub async fn list_dmm_omit_works(&self) -> anyhow::Result<Vec<DmmOmitItem>> {
		let all = self.manager.run(|repos| {
			Box::pin(async move {
				let mut repo = repos.work();
				repo.list_all_details().await
			})
		}).await?;
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

#[derive(Clone, Debug)]
pub struct DmmOmitItem {
	pub work_id: i32,
	pub store_id: String,
	pub category: String,
	pub subcategory: String,
}

