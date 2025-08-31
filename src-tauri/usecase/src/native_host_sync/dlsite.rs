use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DlsiteKey { store_id: String, category: String }

#[derive(Clone, Debug)]
struct DlsiteBatchSnapshot {
	work_id_by_key: HashMap<DlsiteKey, Option<i32>>,
	mapped_keys: HashMap<DlsiteKey, domain::Id<domain::collection::CollectionElement>>,
	omitted_work_ids: HashSet<i32>,
	egs_id_to_collection_id: HashMap<i32, domain::Id<domain::collection::CollectionElement>>,
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
		}

		Ok(DlsiteBatchSnapshot { work_id_by_key, mapped_keys, omitted_work_ids, egs_id_to_collection_id })
	}

	pub async fn sync_dlsite_games(&self, games: Vec<DlsiteSyncGameParam>) -> anyhow::Result<u32> {
		let snapshot = self.build_dlsite_batch_snapshot(&games).await?;
		let resolver = self.resolver.clone();
		self.manager.run_in_transaction(move |repos| {
			let mut success: u32 = 0;
			let mut egs_cache = snapshot.egs_id_to_collection_id.clone();
			let apps = games.clone();
			Box::pin(async move {
				for p in apps.into_iter() {
					let key = DlsiteKey { store_id: p.store_id.clone(), category: p.category.clone() };
					if snapshot.mapped_keys.contains_key(&key) { continue; }
					if let Some(wid) = snapshot.work_id_by_key.get(&key).and_then(|v| *v) { if snapshot.omitted_work_ids.contains(&wid) { continue; } }

					let collection_element_id = match p.egs.as_ref() {
						Some(egs_info) => {
							if let Some(cid) = egs_cache.get(&egs_info.erogamescape_id) { cid.clone() } else {
								let cid = Self::ensure_collection_for_egs_with_repos(&repos, egs_info).await?;
								let _ = egs_cache.insert(egs_info.erogamescape_id, cid.clone());
								cid
							}
						}
						None => { Self::create_collection_without_egs_with_repos(&repos, &p.gamename).await? }
					};

					if let Some(work_id) = snapshot.work_id_by_key.get(&key).and_then(|v| *v) {
						let mut col = repos.collection();
						col.upsert_work_mapping(&collection_element_id, work_id).await?;
					}

					if !p.image_url.is_empty() {
						let icon_dst = resolver.icon_png_path(collection_element_id.value);
						{
							let mut iq = repos.image_queue();
							let _ = iq.enqueue(&p.image_url, ImageSrcType::Url, &icon_dst, ImagePreprocess::ResizeAndCropSquare256).await;
						}
						let normalized = normalize_thumbnail_url(&p.image_url);
						let thumb_dst = resolver.thumbnail_png_path(collection_element_id.value);
						{
							let mut iq = repos.image_queue();
							let _ = iq.enqueue(&normalized, ImageSrcType::Url, &thumb_dst, ImagePreprocess::ResizeForWidth400).await;
						}
						let alias = resolver.thumbnail_alias_dlsite_png_path(&p.category, &p.store_id);
						{
							let mut iq = repos.image_queue();
							let _ = iq.enqueue(&normalized, ImageSrcType::Url, &alias, ImagePreprocess::ResizeForWidth400).await;
						}
					}

					success += 1;
				}
				Ok(success)
			})
		}).await
	}
}
