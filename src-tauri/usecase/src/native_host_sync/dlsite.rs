use domain::Id;

use super::*;
use domain::repository::{works::WorkRepository};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DlsiteKey { store_id: String, category: String }

#[derive(Clone, Debug)]
struct DlsiteBatchSnapshot {
	work_id_by_key: HashMap<DlsiteKey, Option<i32>>,
	mapped_keys: HashMap<DlsiteKey, domain::Id<domain::collection::CollectionElement>>,
	omitted_work_ids: HashSet<i32>,
	egs_id_to_collection_id: HashMap<i32, domain::Id<domain::collection::CollectionElement>>,
	egs_id_to_work_id: HashMap<i32, i32>,
}

#[derive(Clone, Debug)]
struct SyncApply {
	key: DlsiteKey,
	work_id_by_key: Option<i32>,
	work_id_by_erogamescape: Option<i32>,
	collection_element_id_by_erogamescape: Option<domain::Id<domain::collection::CollectionElement>>,
	gamename: String,
	image_url: String,
	egs: Option<EgsInfo>,
}

#[derive(Clone, Debug)]
enum PlanDecision {
	SkipExists,
	SkipOmitted,
	Apply(SyncApply),
}

impl<M, R> NativeHostSyncUseCase<M, R>
where
	M: RepositoryManager<R>,
	R: RepositoriesExt + Send + Sync + 'static,
{
	async fn build_dlsite_batch_snapshot(&self, games: &[DlsiteSyncGameParam]) -> anyhow::Result<DlsiteBatchSnapshot> {
		let keys: Vec<(String, String)> = games.iter().map(|g| (g.store_id.clone(), g.category.clone())).collect();
		let mut work_id_by_key: HashMap<DlsiteKey, Option<i32>> = HashMap::new();
		let found_map: HashMap<(String, String), i32> = self.manager.run(|repos| {
			let keys = keys.clone();
			Box::pin(async move {
				let mut repo = repos.dlsite_work();
				let mut out: HashMap<(String, String), i32> = HashMap::new();
				for (sid, cat) in keys.into_iter() {
					if let Some(w) = repo.find_by_store_key(&sid, &cat).await? {
						out.insert((sid, cat), w.id.value);
					}
				}
				Ok::<_, anyhow::Error>(out)
			})
		}).await?;
		for ((sid, cat), wid) in found_map.into_iter() { work_id_by_key.insert(DlsiteKey { store_id: sid, category: cat }, Some(wid)); }
		for (sid, cat) in keys.iter() { work_id_by_key.entry(DlsiteKey { store_id: sid.clone(), category: cat.clone() }).or_insert(None); }

		let mut mapped_keys: HashMap<DlsiteKey, domain::Id<domain::collection::CollectionElement>> = HashMap::new();
		let work_ids: Vec<i32> = keys.iter().filter_map(|(sid, cat)| work_id_by_key.get(&DlsiteKey { store_id: sid.clone(), category: cat.clone() }).and_then(|v| *v)).collect();
		if !work_ids.is_empty() {
			let rows = self.manager.run(|repos| {
				let work_ids = work_ids.clone();
				Box::pin(async move {
					let mut repo = repos.collection();
					repo.get_collection_ids_by_work_ids(&work_ids).await
				})
			}).await?;
			let mut keys_by_work: HashMap<i32, Vec<DlsiteKey>> = HashMap::new();
			for (k, v) in work_id_by_key.iter() { if let Some(wid) = v { keys_by_work.entry(*wid).or_default().push(k.clone()); } }
			for (wid, ce) in rows.into_iter() { if let Some(kk) = keys_by_work.get(&wid) { for k in kk.iter() { mapped_keys.insert(k.clone(), ce.clone()); } } }
		}

		let omitted_work_ids: HashSet<i32> = self.manager.run(|repos| {
			Box::pin(async move {
				let mut repo = repos.work_omit();
				let list = repo.list().await?;
				Ok(list.into_iter().map(|o| o.work_id.value).collect())
			})
		}).await?;

		let mut egs_id_to_collection_id: HashMap<i32, domain::Id<domain::collection::CollectionElement>> = HashMap::new();
		let mut egs_id_to_work_id: HashMap<i32, i32> = HashMap::new();
		let egs_ids: Vec<i32> = games.iter().filter_map(|g| g.egs.as_ref().map(|e| e.erogamescape_id)).collect();
		if !egs_ids.is_empty() {
			let rows = self.manager.run(|repos| {
				let egs_ids = egs_ids.clone();
				Box::pin(async move {
					let mut repo = repos.collection();
					repo.get_collection_ids_by_erogamescape_ids(&egs_ids).await
				})
			}).await?;
			for (egs_id, ceid) in rows.into_iter() { egs_id_to_collection_id.insert(egs_id, ceid); }

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
				for (ceid, wid) in rows.into_iter() { first_work_by_ce.entry(ceid.value).or_insert(wid); }
				for (egs_id, ceid) in egs_id_to_collection_id.iter() {
					if let Some(wid) = first_work_by_ce.get(&ceid.value) { egs_id_to_work_id.insert(*egs_id, *wid); }
				}
			}
		}

		Ok(DlsiteBatchSnapshot { work_id_by_key, mapped_keys, omitted_work_ids, egs_id_to_collection_id, egs_id_to_work_id })
	}

	async fn decide_for_dlsite_game(&self, snapshot: &DlsiteBatchSnapshot, param: DlsiteSyncGameParam) -> anyhow::Result<PlanDecision> {
		let key = DlsiteKey { store_id: param.store_id.clone(), category: param.category.clone() };

		if snapshot.mapped_keys.contains_key(&key) { return Ok(PlanDecision::SkipExists); }

		let work_id = snapshot.work_id_by_key.get(&key).cloned().unwrap_or(None);
		if let Some(work_id) = work_id {
			if snapshot.omitted_work_ids.contains(&work_id) { return Ok(PlanDecision::SkipOmitted); }
		}

		Ok(PlanDecision::Apply(SyncApply {
			key,
			work_id_by_key: work_id,
			work_id_by_erogamescape: param.egs.as_ref().and_then(|e| snapshot.egs_id_to_work_id.get(&e.erogamescape_id).cloned()),
			collection_element_id_by_erogamescape: param.egs.as_ref().and_then(|e| snapshot.egs_id_to_collection_id.get(&e.erogamescape_id).cloned()),
			gamename: param.gamename,
			image_url: param.image_url,
			egs: param.egs,
		}))
	}

	pub async fn sync_dlsite_games(&self, games: Vec<DlsiteSyncGameParam>) -> anyhow::Result<u32> {
		let snapshot = self.build_dlsite_batch_snapshot(&games).await?;
		let mut plans: Vec<PlanDecision> = Vec::with_capacity(games.len());
		for param in games.iter().cloned() { plans.push(self.decide_for_dlsite_game(&snapshot, param).await?); }
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
							Self::execute_apply_dlsite_with_repos(&repos, apply, resolver.as_ref()).await?;
							success += 1;
						}
					}
				}
				Ok(success)
			})
		}).await
	}

	pub(crate) async fn enqueue_images_for_dlsite_with_repos<Rx: RepositoriesExt + Send + Sync + 'static>(
		repos: &Rx,
		resolver: &dyn SavePathResolver,
		collection_element_id: &domain::Id<domain::collection::CollectionElement>,
		category: &str,
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

		let alias = resolver.thumbnail_alias_dlsite_png_path(category, store_id);
		{
			let mut repo = repos.image_queue();
			let _ = repo.enqueue(&normalized, ImageSrcType::Url, &alias, ImagePreprocess::ResizeForWidth400).await;
		}

		Ok(())
	}

	async fn execute_apply_dlsite_with_repos<Rx: RepositoriesExt + Send + Sync + 'static>(
		repos: &Rx,
		apply: SyncApply,
		resolver: &dyn SavePathResolver,
	) -> anyhow::Result<()> {
		let SyncApply { key, work_id_by_key, work_id_by_erogamescape, collection_element_id_by_erogamescape, gamename, image_url, egs } = apply;

		// Work を用意
		let work_id = match work_id_by_key.or(work_id_by_erogamescape) {
			Some(work_id) => Id::new(work_id),
			None => repos.work().upsert(&domain::works::NewWork::new(gamename.clone())).await?,
		};
		// DLsite キーへのマッピング upsert
		repos.dlsite_work().upsert(&domain::works::NewDlsiteWork::new(key.store_id.clone(), key.category.clone(), work_id.clone())).await?;

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

		// CE-Work マッピング
		repos.collection().upsert_work_mapping(&collection_element_id, work_id.value).await?;

		// 画像投入
		Self::enqueue_images_for_dlsite_with_repos(repos, resolver, &collection_element_id, &key.category, &key.store_id, &image_url).await?;

		Ok(())
	}
}
