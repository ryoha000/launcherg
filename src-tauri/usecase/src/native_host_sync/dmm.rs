use super::*;

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
}

#[derive(Clone, Debug)]
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
			work_id_opt: work_id,
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
		let _ = self.manager.run(|repos| {
			let image_url = image_url.to_string();
			let icon_dst = icon_dst.clone();
			Box::pin(async move {
				let mut repo = repos.image_queue();
				let _ = repo.enqueue(&image_url, ImageSrcType::Url, &icon_dst, ImagePreprocess::ResizeAndCropSquare256).await;
				Ok(())
			})
		}).await;

		let normalized = normalize_thumbnail_url(image_url);
		let thumb_dst = self.resolver.thumbnail_png_path(collection_element_id.value);
		let _ = self.manager.run(|repos| {
			let normalized = normalized.clone();
			let thumb_dst = thumb_dst.clone();
			Box::pin(async move {
				let mut repo = repos.image_queue();
				let _ = repo.enqueue(&normalized, ImageSrcType::Url, &thumb_dst, ImagePreprocess::ResizeForWidth400).await;
				Ok(())
			})
		}).await;

		let alias = self.resolver.thumbnail_alias_dmm_png_path(category, subcategory, store_id);
		let _ = self.manager.run(|repos| {
			let normalized = normalized.clone();
			let alias = alias.clone();
			Box::pin(async move {
				let mut repo = repos.image_queue();
				let _ = repo.enqueue(&normalized, ImageSrcType::Url, &alias, ImagePreprocess::ResizeForWidth400).await;
				Ok(())
			})
		}).await;

		Ok(())
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
			let collection_element_id_cloned = collection_element_id.clone();
			self.manager.run(|repos| Box::pin(async move {
				let mut repo = repos.collection();
				repo.upsert_work_mapping(&collection_element_id_cloned, work_id).await?;
				if let Some(pid) = parent_pack_work_id {
					let _ = repos.work_parent_packs().add(domain::Id::new(work_id), domain::Id::new(pid)).await;
				}
				Ok(())
			})).await?;
		}

		// 画像投入（失敗は握りつぶして継続する実装方針を踏襲）
		let _ = self.enqueue_images_for_dmm(&collection_element_id, &key.category, &key.subcategory, &key.store_id, &image_url).await;

		Ok(())
	}

	/// 計画にもとづき副作用を実行（トランザクション内のリポジトリを使用）
	async fn execute_apply_with_repos<Rx: RepositoriesExt + Send + Sync + 'static>(
		repos: &Rx,
		apply: SyncApply,
		caches: &mut Caches,
		resolver: &dyn SavePathResolver,
	) -> anyhow::Result<()> {
		let SyncApply { key, work_id_opt, gamename, image_url, parent_pack_work_id, egs } = apply;

		// Collection Element を用意
		let collection_element_id = match egs.as_ref() {
			Some(egs_info) => {
				if let Some(cid) = caches.egs_id_to_collection_id.get(&egs_info.erogamescape_id) {
					cid.clone()
				} else {
					let cid = Self::ensure_collection_for_egs_with_repos(repos, egs_info).await?;
					caches.egs_id_to_collection_id.insert(egs_info.erogamescape_id, cid.clone());
					cid
				}
			}
			None => {
				Self::create_collection_without_egs_with_repos(repos, &gamename).await?
			}
		};

		// Work があればマッピング/親子リンク
		if let Some(work_id) = work_id_opt {
			let mut col = repos.collection();
			col.upsert_work_mapping(&collection_element_id, work_id).await?;
			if let Some(pid) = parent_pack_work_id {
				let mut pprepo = repos.work_parent_packs();
				let _ = pprepo.add(domain::Id::new(work_id), domain::Id::new(pid)).await;
			}
		}

		// 画像投入（失敗は握りつぶして継続）
		let _ = Self::enqueue_images_for_dmm_with_repos(repos, resolver, &collection_element_id, &key.category, &key.subcategory, &key.store_id, &image_url).await;

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
			let mut caches = Caches { egs_id_to_collection_id: snapshot.egs_id_to_collection_id.clone() };
			let plans = plans.clone();
			let resolver = resolver.clone();
			Box::pin(async move {
				let mut success: u32 = 0;
				for plan in plans.into_iter() {
					match plan {
						PlanDecision::SkipExists | PlanDecision::SkipOmitted => {}
						PlanDecision::Apply(apply) => {
							Self::execute_apply_with_repos(&repos, apply, &mut caches, resolver.as_ref()).await?;
							success += 1;
						}
					}
				}
				Ok(success)
			})
		}).await
	}
}


