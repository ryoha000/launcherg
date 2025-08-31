use domain::Id;

use super::*;
use domain::repository::{works::WorkRepository};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DmmKey {
	store_id: String,
	category: String,
	subcategory: String,
}

#[derive(Clone, Debug)]
struct DmmBatchSnapshot {
	work_id_by_key: HashMap<DmmKey, Option<i32>>,
	mapped_keys: HashMap<DmmKey, domain::Id<domain::collection::CollectionElement>>,
	omitted_work_ids: HashSet<i32>,
	egs_id_to_collection_id: HashMap<i32, domain::Id<domain::collection::CollectionElement>>,
	egs_id_to_work_id: HashMap<i32, i32>,
}

#[derive(Clone, Debug)]
struct SyncApply {
	key: DmmKey,
	work_id_by_key: Option<i32>,
	work_id_by_erogamescape: Option<i32>,
	collection_element_id_by_erogamescape: Option<domain::Id<domain::collection::CollectionElement>>,
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

impl<M, R> NativeHostSyncUseCase<M, R>
where
	M: RepositoryManager<R>,
	R: RepositoriesExt + Send + Sync + 'static,
{
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
			work_id_by_key: work_id,
			work_id_by_erogamescape: param
				.egs
				.as_ref()
				.and_then(|e| snapshot.egs_id_to_work_id.get(&e.erogamescape_id).cloned()),
			collection_element_id_by_erogamescape: param
				.egs
				.as_ref()
				.and_then(|e| snapshot.egs_id_to_collection_id.get(&e.erogamescape_id).cloned()),
			gamename: param.gamename,
			image_url: param.image_url,
			parent_pack_work_id: param.parent_pack_work_id,
			egs: param.egs,
		}))
	}
}

impl<M, R> NativeHostSyncUseCase<M, R>
where
	M: RepositoryManager<R>,
	R: RepositoriesExt + Send + Sync + 'static,
{
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

		// Work の取得（1キーずつ lookup して対応関係を構築）
		let mut work_id_by_key: HashMap<DmmKey, Option<i32>> = HashMap::new();
		let found_map: HashMap<(String, String, String), i32> = self.manager.run(|repos| {
			let keys = keys.clone();
			Box::pin(async move {
				let mut repo = repos.dmm_work();
				let mut out: HashMap<(String, String, String), i32> = HashMap::new();
				for (sid, cat, sub) in keys.into_iter() {
					if let Some(w) = repo.find_by_store_key(&sid, &cat, &sub).await? {
						out.insert((sid, cat, sub), w.id.value);
					}
				}
				Ok::<_, anyhow::Error>(out)
			})
		}).await?;
		for ((sid, cat, sub), wid) in found_map.into_iter() {
			work_id_by_key.insert(DmmKey { store_id: sid, category: cat, subcategory: sub }, Some(wid));
		}
		// 入力に対して必ずキーを用意（未取得は None）
		for (sid, cat, sub) in keys.iter() {
			work_id_by_key.entry(DmmKey { store_id: sid.clone(), category: cat.clone(), subcategory: sub.clone() }).or_insert(None);
		}

		// 既存マッピングの一括取得（work_id 直引き）
		let mut mapped_keys: HashMap<DmmKey, domain::Id<domain::collection::CollectionElement>> = HashMap::new();
		// HashMap の列挙順に依存しないよう、入力順 `keys` に基づいて work_ids を構築する
		let work_ids: Vec<i32> = keys
			.iter()
			.filter_map(|(sid, cat, sub)| {
				work_id_by_key
					.get(&DmmKey { store_id: sid.clone(), category: cat.clone(), subcategory: sub.clone() })
					.and_then(|v| *v)
			})
			.collect();
		// work_ids は多くなる可能性があるためチャンクして問い合わせ
		let mut existing: Vec<(i32, domain::Id<domain::collection::CollectionElement>)> = Vec::new();
		let mut woff = 0usize;
		let wchunk = 200usize; // パラメータ上限を考慮
		while woff < work_ids.len() {
			let wend = (woff + wchunk).min(work_ids.len());
			let sub = work_ids[woff..wend].to_vec();
			let rows = self.manager.run(|repos| {
				let sub = sub.clone();
				Box::pin(async move {
					let mut repo = repos.collection();
					repo.get_collection_ids_by_work_ids(&sub).await
				})
			}).await?;
			existing.extend(rows.into_iter());
			woff = wend;
		}
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

		// omit 一括取得
		let omitted_work_ids: HashSet<i32> = self.manager.run(|repos| {
			Box::pin(async move {
				let mut repo = repos.work_omit();
				let list = repo.list().await?;
				Ok(list.into_iter().map(|o| o.work_id.value).collect())
			})
		}).await?;

		// EGS 一括取得
		let mut egs_id_to_collection_id: HashMap<i32, domain::Id<domain::collection::CollectionElement>> = HashMap::new();
		let mut egs_id_to_work_id: HashMap<i32, i32> = HashMap::new();
		let egs_ids: Vec<i32> = games
			.iter()
			.filter_map(|g| g.egs.as_ref().map(|e| e.erogamescape_id))
			.collect();
		if !egs_ids.is_empty() {
			let rows = self.manager.run(|repos| {
				let egs_ids = egs_ids.clone();
				Box::pin(async move {
					let mut repo = repos.collection();
					repo.get_collection_ids_by_erogamescape_ids(&egs_ids).await
				})
			}).await?;
			for (egs_id, ceid) in rows.into_iter() { egs_id_to_collection_id.insert(egs_id, ceid); }

			// CE -> Work の逆引きで EGS -> Work を用意
			let ce_ids: Vec<i32> = egs_id_to_collection_id.values().map(|id| id.value).collect();
			if !ce_ids.is_empty() {
				let rows = self.manager.run(|repos| {
					let ce_ids = ce_ids.clone();
					Box::pin(async move {
						let mut repo = repos.collection();
						repo.get_work_ids_by_collection_ids(&ce_ids).await
					})
				}).await?;
				let mut first_work_by_ce: HashMap<i32, i32> = HashMap::new();
				for (ceid, wid) in rows.into_iter() {
					first_work_by_ce.entry(ceid.value).or_insert(wid);
				}
				for (egs_id, ceid) in egs_id_to_collection_id.iter() {
					if let Some(wid) = first_work_by_ce.get(&ceid.value) {
						egs_id_to_work_id.insert(*egs_id, *wid);
					}
				}
			}
		}

		Ok(DmmBatchSnapshot { work_id_by_key, mapped_keys, omitted_work_ids, egs_id_to_collection_id, egs_id_to_work_id })
	}

	/// 作品画像をキュー投入する（トランザクション内で repos を直接使用）
	pub(crate) async fn enqueue_images_for_dmm_with_repos<Rx: RepositoriesExt + Send + Sync + 'static>(
		repos: &Rx,
		resolver: &dyn SavePathResolver,
		collection_element_id: &domain::Id<domain::collection::CollectionElement>,
		category: &str,
		subcategory: &str,
		store_id: &str,
		image_url: &str,
	) -> anyhow::Result<()> {
		if image_url.is_empty() { return Ok(()); }

		let icon_dst = resolver.icon_png_path(collection_element_id.value);
		{
			let mut repo = repos.image_queue();
			let _ = repo.enqueue(image_url, ImageSrcType::Url, &icon_dst, ImagePreprocess::ResizeAndCropSquare256).await;
		}

		let normalized = normalize_thumbnail_url(image_url);
		let thumb_dst = resolver.thumbnail_png_path(collection_element_id.value);
		{
			let mut repo = repos.image_queue();
			let _ = repo.enqueue(&normalized, ImageSrcType::Url, &thumb_dst, ImagePreprocess::ResizeForWidth400).await;
		}

		let alias = resolver.thumbnail_alias_dmm_png_path(category, subcategory, store_id);
		{
			let mut repo = repos.image_queue();
			let _ = repo.enqueue(&normalized, ImageSrcType::Url, &alias, ImagePreprocess::ResizeForWidth400).await;
		}

		Ok(())
	}

	/// 計画にもとづき副作用を実行（トランザクション内のリポジトリを使用）
	async fn execute_apply_with_repos<Rx: RepositoriesExt + Send + Sync + 'static>(
		repos: &Rx,
		apply: SyncApply,
		resolver: &dyn SavePathResolver,
	) -> anyhow::Result<()> {
		let SyncApply { key, work_id_by_key, work_id_by_erogamescape, collection_element_id_by_erogamescape, gamename, image_url, parent_pack_work_id, egs } = apply;

        // work を用意
        let work_id = match work_id_by_key.or(work_id_by_erogamescape) {
            Some(work_id) => Id::new(work_id),
            None => {
                repos.work().upsert(&domain::works::NewWork::new(gamename.clone())).await?
            }
        };

        // key に work_id をマッピング
        repos.dmm_work().upsert(&domain::works::NewDmmWork::new(key.store_id.clone(), key.category.clone(), key.subcategory.clone(), work_id.clone())).await?;

		// Collection Element を用意
        let collection_element_id = match collection_element_id_by_erogamescape {
            Some(ceid) => ceid,
            None => {
                let ceid = repos.collection().allocate_new_collection_element_id(&gamename).await?;
                if let Some(egs_info) = egs.as_ref() {
                    repos.collection().upsert_erogamescape_map(&ceid, egs_info.erogamescape_id).await?;
                    repos.collection().upsert_collection_element_info(&domain::collection::NewCollectionElementInfo::new(ceid.clone(), gamename.clone(), egs_info.brandname.clone(), egs_info.brandname_ruby.clone(), egs_info.sellday.clone(), egs_info.is_nukige)).await?;
                }
                ceid
            }
        };

        // collection_element に work_id をマッピング
        repos.collection().upsert_work_mapping(&collection_element_id, work_id.value).await?;

		// pack の親が設定されていればマッピング
		if let Some(pid) = parent_pack_work_id {
            repos.work_parent_packs().add(work_id, domain::Id::new(pid)).await?;
        }

		// 画像投入
		Self::enqueue_images_for_dmm_with_repos(repos, resolver, &collection_element_id, &key.category, &key.subcategory, &key.store_id, &image_url).await?;

		Ok(())
	}

	// ensure_collection_for_egs と create_collection_without_egs は mod.rs に保持

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
		let snapshot = self.build_dmm_batch_snapshot(&games).await?;
		let mut plans: Vec<PlanDecision> = Vec::with_capacity(games.len());
		for param in games.into_iter() { plans.push(self.decide_for_game(&snapshot, param).await?); }
		let resolver = self.resolver.clone();
		self.manager.run_in_transaction(move |repos| {
			let plans = plans.clone();
			let resolver = resolver.clone();
			Box::pin(async move {
				let mut success: u32 = 0;
				for plan in plans.into_iter() {
					match plan {
						PlanDecision::SkipExists | PlanDecision::SkipOmitted => {}
						PlanDecision::Apply(apply) => {
							Self::execute_apply_with_repos(&repos, apply, resolver.as_ref()).await?;
							success += 1;
						}
					}
				}
				Ok(success)
			})
		}).await
	}
}


