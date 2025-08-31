//! ネイティブホスト同期ユースケース: ストア(DMM/DLsite)のゲーム情報をコレクションへ同期する。
//! - 既存マッピングがあればスキップして冪等性を保つ
//! - EGS 情報があれば名称/詳細も upsert し、EGS マップを作成/更新する

use std::sync::Arc;
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
mod store;

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
			if w.is_omitted {
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

