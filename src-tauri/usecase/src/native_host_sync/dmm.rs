use super::*;
use super::store::{StoreOps, PlanDecisionGeneric};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DmmKey {
	store_id: String,
	category: String,
	subcategory: String,
}

// 旧スナップショット/適用関連は共通パイプラインへ移行済み

impl DmmKey {
	fn from_param(p: &DmmSyncGameParam) -> DmmKey {
		DmmKey { store_id: p.store_id.clone(), category: p.category.clone(), subcategory: p.subcategory.clone() }
	}
}

impl<M, R> NativeHostSyncUseCase<M, R>
where
	M: RepositoryManager<R>,
	R: RepositoriesExt + Send + Sync + 'static,
{
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
		// 構成子
		fn key_from_param(p: &DmmSyncGameParam) -> DmmKey { DmmKey::from_param(p) }
		fn gamename(p: &DmmSyncGameParam) -> &str { &p.gamename }
		fn image_url(p: &DmmSyncGameParam) -> &str { &p.image_url }
		fn egs(p: &DmmSyncGameParam) -> Option<&EgsInfo> { p.egs.as_ref() }
		fn parent_pack_work_id(p: &DmmSyncGameParam) -> Option<i32> { p.parent_pack_work_id }

		fn find_work_id_by_key<'a, Rx: RepositoriesExt + Send + Sync + 'static>(repos: &'a Rx, k: &'a DmmKey) -> futures::future::BoxFuture<'a, anyhow::Result<Option<i32>>> {
			Box::pin(async move {
				let mut repo = repos.dmm_work();
				Ok(repo.find_by_store_key(&k.store_id, &k.category, &k.subcategory).await?.map(|w| w.id.value))
			})
		}
		fn upsert_store_mapping<'a, Rx: RepositoriesExt + Send + Sync + 'static>(repos: &'a Rx, k: &'a DmmKey, work_id: domain::Id<domain::works::Work>) -> futures::future::BoxFuture<'a, anyhow::Result<()>> {
			Box::pin(async move {
				let mut repo = repos.dmm_work();
				let _ = repo.upsert(&domain::works::NewDmmWork::new(k.store_id.clone(), k.category.clone(), k.subcategory.clone(), work_id)).await?;
				Ok(())
			})
		}
		fn link_parent_pack_if_needed<'a, Rx: RepositoriesExt + Send + Sync + 'static>(repos: &'a Rx, work_id: domain::Id<domain::works::Work>, parent: Option<i32>) -> futures::future::BoxFuture<'a, anyhow::Result<()>> {
			Box::pin(async move {
				if let Some(pid) = parent {
					repos.work_parent_packs().add(work_id, domain::Id::new(pid)).await?;
				}
				Ok(())
			})
		}
		fn enqueue_images_with_repos<'a, Rx: RepositoriesExt + Send + Sync + 'static>(repos: &'a Rx, resolver: &'a dyn SavePathResolver, ceid: &'a domain::Id<domain::collection::CollectionElement>, key: &'a DmmKey, image_url: &'a str) -> futures::future::BoxFuture<'a, anyhow::Result<()>> {
			Box::pin(async move {
				if image_url.is_empty() { return Ok(()); }
				let icon_dst = resolver.icon_png_path(ceid.value);
				{
					let mut repo = repos.image_queue();
					let _ = repo.enqueue(image_url, ImageSrcType::Url, &icon_dst, ImagePreprocess::ResizeAndCropSquare256).await;
				}
				let normalized = normalize_thumbnail_url(image_url);
				let thumb_dst = resolver.thumbnail_png_path(ceid.value);
				{
					let mut repo = repos.image_queue();
					let _ = repo.enqueue(&normalized, ImageSrcType::Url, &thumb_dst, ImagePreprocess::ResizeForWidth400).await;
				}
				let alias = resolver.thumbnail_alias_dmm_png_path(&key.category, &key.subcategory, &key.store_id);
				{
					let mut repo = repos.image_queue();
					let _ = repo.enqueue(&normalized, ImageSrcType::Url, &alias, ImagePreprocess::ResizeForWidth400).await;
				}
				Ok(())
			})
		}

		let ops: StoreOps<DmmSyncGameParam, DmmKey, R> = StoreOps {
			key_from_param,
			gamename,
			image_url,
			egs,
			parent_pack_work_id,
			find_work_id_by_key: find_work_id_by_key::<R>,
			upsert_store_mapping: upsert_store_mapping::<R>,
			enqueue_images_with_repos: enqueue_images_with_repos::<R>,
			link_parent_pack_if_needed: link_parent_pack_if_needed::<R>,
		};

		let snapshot = self.build_batch_snapshot(&games, &ops).await?;
		let mut plans: Vec<PlanDecisionGeneric<DmmKey>> = Vec::with_capacity(games.len());
		for param in games.into_iter() { plans.push(self.decide_for_game_generic(&snapshot, param, &ops).await?); }
		let resolver = self.resolver.clone();
		self.manager.run_in_transaction(move |repos| {
			let plans = plans.clone();
			let resolver = resolver.clone();
			let ops = ops;
			Box::pin(async move {
				let mut success: u32 = 0;
				for plan in plans.into_iter() {
					match plan {
						PlanDecisionGeneric::SkipExists | PlanDecisionGeneric::SkipOmitted => {}
						PlanDecisionGeneric::Apply(apply) => {
							Self::execute_apply_with_repos_generic(&repos, apply, resolver.as_ref(), &ops).await?;
							success += 1;
						}
					}
				}
				Ok(success)
			})
		}).await
	}
}


