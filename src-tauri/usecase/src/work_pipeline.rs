use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI32, Ordering};
use std::{marker::PhantomData, sync::Arc};

use domain::collection::CollectionElement;
use domain::pubsub::{DedupResultPayload, EnrichResultPayload, PubSubEvent, PubSubService};
use domain::repository::{
    all_game_cache::AllGameCacheRepository as _, collection::CollectionRepository as _,
    explored_cache::ExploredCacheRepository as _, manager::RepositoryManager,
    save_image_queue::ImageSaveQueueRepository as _, works::WorkRepository as _, RepositoriesExt,
};
use domain::save_image_queue::{ImagePreprocess, ImageSrcType};
use domain::scan::{
    CandidateKind, DuplicateResolver, FileSystem, MetadataExtractor, ResolvedWork, WorkCandidate,
    WorkCandidateOrResolvedWork,
};
use domain::service::save_path_resolver::SavePathResolver;
use domain::service::work_linker::{WorkLinkTask, WorkLinker};
use domain::Id;
use futures::StreamExt as _;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct WorkPipelineUseCase<M, R, P, FS, ME, DR, WL>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService,
    FS: FileSystem,
    ME: MetadataExtractor,
    DR: DuplicateResolver,
    WL: WorkLinker,
{
    manager: Arc<M>,
    pubsub: P,
    fs: Arc<FS>,
    extractor: Arc<ME>,
    dedup: Arc<DR>,
    resolver: Arc<dyn SavePathResolver>,
    linker: Arc<WL>,
    _marker: PhantomData<R>,
}

impl<M, R, P, FS, ME, DR, WL> WorkPipelineUseCase<M, R, P, FS, ME, DR, WL>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService,
    FS: FileSystem,
    ME: MetadataExtractor + Send + Sync + 'static,
    DR: DuplicateResolver,
    WL: WorkLinker + Send + Sync + 'static,
{
    pub fn new(
        manager: Arc<M>,
        pubsub: P,
        fs: Arc<FS>,
        extractor: Arc<ME>,
        dedup: Arc<DR>,
        resolver: Arc<dyn SavePathResolver>,
        linker: Arc<WL>,
    ) -> Self {
        Self {
            manager,
            pubsub,
            fs,
            extractor,
            dedup,
            resolver,
            linker,
            _marker: PhantomData,
        }
    }

    pub async fn start(
        &self,
        roots: Vec<std::path::PathBuf>,
        use_cache: bool,
    ) -> anyhow::Result<Vec<String>> {
        let rx = self.open_candidate_stream(&roots, use_cache).await?;
        // フェーズ: メタ付与（並列・ストリーミング）
        let (resolved, explored, _processed_count) = self
            .enrich_candidates_parallel_stream(ReceiverStream::new(rx))
            .await;

        // フェーズ: 重複排除
        let recognized_len = resolved.len();
        let (deduped, _duplicates) = self.deduplicate_and_notify(resolved, recognized_len);

        // フェーズ: 永続化
        let _ = self.persist(&deduped).await?;
        // フェーズ: .lnk 保全
        let link_tasks = self.prepare_link_tasks(&deduped).await?;
        self.linker.ensure_links(link_tasks).await?;
        self.update_explored_cache(explored).await?;

        self.backfill_thumbnail_sizes().await?;
        Ok(deduped.into_iter().map(|r| r.title).collect())
    }

    pub(crate) async fn open_candidate_stream(
        &self,
        roots: &[std::path::PathBuf],
        use_cache: bool,
    ) -> anyhow::Result<mpsc::Receiver<WorkCandidate>> {
        let exclude = if use_cache {
            let cache = self
                .manager
                .run(|repos| {
                    Box::pin(async move {
                        Ok::<_, anyhow::Error>(repos.explored_cache().get_all().await?)
                    })
                })
                .await?;
            Some(std::sync::Arc::new(cache))
        } else {
            None
        };

        let iter = self.fs.walk_dir(roots, exclude)?;

        let (tx, rx) = mpsc::channel::<WorkCandidate>(2048);
        tokio::spawn(async move {
            for c in iter {
                match tx.try_send(c) {
                    Ok(_) => {}
                    Err(tokio::sync::mpsc::error::TrySendError::Full(c)) => {
                        if tx.send(c).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => break,
                }
            }
        });
        Ok(rx)
    }

    pub(crate) async fn enrich_candidates_parallel_stream<S>(
        &self,
        candidates: S,
    ) -> (Vec<ResolvedWork>, Vec<String>, usize)
    where
        S: futures::Stream<Item = WorkCandidate> + Unpin,
    {
        let extractor = self.extractor.clone();
        let processed = std::sync::Arc::new(AtomicI32::new(0));
        let pubsub_en = &self.pubsub;
        let processed_counter = processed.clone();
        // 並列度は CPUスレッド×4（最大512）で固定
        let default_parallel = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let parallel_count = (default_parallel * 4).max(1).min(512);
        let enriched = candidates
            .map(|c| {
                let extractor = extractor.clone();
                // CPUバウンド処理はblockingプールへ
                tokio::task::spawn_blocking(move || extractor.enrich(c))
            })
            .buffer_unordered(parallel_count)
            .filter_map(move |res| {
                let pubsub = pubsub_en;
                let processed = processed_counter.clone();
                async move {
                    let _ = processed.fetch_add(1, Ordering::Relaxed) + 1;
                    match res {
                        Ok(Ok(v)) => {
                            match &v {
                                WorkCandidateOrResolvedWork::Candidate(c) => {
                                    let path = c.path.to_string_lossy().to_string();
                                    let _ = pubsub.notify(PubSubEvent::ScanEnrichResult(
                                        EnrichResultPayload::new(
                                            "candidate".into(),
                                            path,
                                            None,
                                            None,
                                        ),
                                    ));
                                }
                                WorkCandidateOrResolvedWork::Resolved(r) => {
                                    let path = r.candidate.path.to_string_lossy().to_string();
                                    let title = r.title.clone();
                                    let egs_id = r.egs_id;
                                    let _ = pubsub.notify(PubSubEvent::ScanEnrichResult(
                                        EnrichResultPayload::new(
                                            "resolved".into(),
                                            path,
                                            Some(title),
                                            Some(egs_id),
                                        ),
                                    ));
                                }
                            }
                            Some(v)
                        }
                        Ok(Err(_e)) => None,
                        Err(_join_err) => None,
                    }
                }
            })
            .collect::<Vec<_>>()
            .await;
        let explored = enriched
            .iter()
            .map(|v| match v {
                WorkCandidateOrResolvedWork::Candidate(c) => c.path.to_string_lossy().to_string(),
                WorkCandidateOrResolvedWork::Resolved(r) => {
                    r.candidate.path.to_string_lossy().to_string()
                }
            })
            .collect();
        let results = enriched
            .into_iter()
            .filter_map(|v| match v {
                WorkCandidateOrResolvedWork::Candidate(_) => None,
                WorkCandidateOrResolvedWork::Resolved(r) => Some(r),
            })
            .collect();
        let count = processed.load(Ordering::Relaxed) as usize;
        (results, explored, count)
    }

    pub(crate) fn deduplicate_and_notify(
        &self,
        resolved: Vec<ResolvedWork>,
        recognized_len: usize,
    ) -> (Vec<ResolvedWork>, usize) {
        let deduped: Vec<ResolvedWork> = self.dedup.resolve(resolved);
        let duplicates = recognized_len.saturating_sub(deduped.len());
        let _ = self
            .pubsub
            .notify(PubSubEvent::ScanDedup(DedupResultPayload::new(
                duplicates as i32,
            )));
        (deduped, duplicates)
    }

    pub(crate) async fn persist(&self, deduped: &[ResolvedWork]) -> anyhow::Result<usize> {
        use std::collections::{HashMap, HashSet};

        let resolver = self.resolver.clone();

        // 単一トランザクションで全件処理
        let count = self
            .manager
            .run_in_transaction(move |repos| {
                let resolver = resolver.clone();
                Box::pin(async move {
                    // 先読みキャッシュを準備（EGS -> CE、CE -> Work）
                    let mut coll = repos.collection();
                    let mut work_repo = repos.work();
                    let resolver = resolver.clone();

                    let mut uniq_egs: Vec<i32> = Vec::new();
                    let mut seen: HashSet<i32> = HashSet::new();
                    for it in deduped.iter() {
                        if seen.insert(it.egs_id) {
                            uniq_egs.push(it.egs_id);
                        }
                    }

                    let mut egs_to_ce: HashMap<i32, Id<CollectionElement>> = HashMap::new();
                    if !uniq_egs.is_empty() {
                        let egs_ce_pairs = coll
                            .get_collection_ids_by_erogamescape_ids(&uniq_egs)
                            .await?; // Vec<(egs, ce)>
                        for (egs, ce) in egs_ce_pairs.into_iter() {
                            egs_to_ce.insert(egs, ce);
                        }
                    }

                    let mut ce_to_work: HashMap<Id<CollectionElement>, Id<domain::works::Work>> =
                        HashMap::new();
                    let ce_ids: Vec<Id<CollectionElement>> = egs_to_ce.values().cloned().collect();
                    if !ce_ids.is_empty() {
                        let ce_work_pairs = coll.get_work_ids_by_collection_ids(&ce_ids).await?; // Vec<(ce, work)>
                        for (ce, w) in ce_work_pairs.into_iter() {
                            ce_to_work.insert(ce, w);
                        }
                    }

                    // 先読み: EGS -> AllGameCacheOneWithThumbnailUrl
                    let mut egs_to_agc: HashMap<
                        i32,
                        domain::all_game_cache::AllGameCacheOneWithThumbnailUrl,
                    > = HashMap::new();
                    if !uniq_egs.is_empty() {
                        let list = repos.all_game_cache().get_by_ids(uniq_egs.clone()).await?;
                        for gc in list.into_iter() {
                            egs_to_agc.insert(gc.id, gc);
                        }
                    }

                    let mut processed = 0usize;
                    for item in deduped.iter() {
                        let title = item.title.clone();
                        let item_file_path = item.candidate.path.to_string_lossy().to_string();

                        // CollectionElement の存在確認（EGS マッピングで判定）
                        let collection_id: Id<CollectionElement> = if let Some(cid) =
                            egs_to_ce.get(&item.egs_id).cloned()
                        {
                            // 既存: 非 upsert の名称更新
                            coll.update_collection_element_gamename_by_id(&cid, &title)
                                .await?;
                            cid
                        } else {
                            // 未存在: 採番（挿入）→ EGS マッピング作成
                            let new_id = coll.allocate_new_collection_element_id(&title).await?;
                            coll.upsert_erogamescape_map(&new_id, item.egs_id).await?;
                            egs_to_ce.insert(item.egs_id, new_id.clone());
                            new_id
                        };

                        // Work の存在確認（CE -> Work マップ）
                        if !ce_to_work.contains_key(&collection_id) {
                            let work_id = work_repo
                                .upsert(&domain::works::NewWork::new(title.clone()))
                                .await?;
                            coll.insert_work_mapping(&collection_id, work_id.clone())
                                .await?;
                            ce_to_work.insert(collection_id.clone(), work_id);
                        }

                        // 画像保存キュー投入（既存ロジックを維持）
                        let mut iq = repos.image_queue();
                        if let Some(exec) = std::path::Path::new(&item_file_path).to_str() {
                            let icon_dst = resolver.icon_png_path(collection_id.value);
                            let src_type = match item.candidate.kind {
                                CandidateKind::Exe => ImageSrcType::Exe,
                                CandidateKind::Shortcut => ImageSrcType::Shortcut,
                                CandidateKind::Folder => anyhow::bail!("folder is not supported"),
                                CandidateKind::Other => anyhow::bail!("other is not supported"),
                            };
                            iq.enqueue(
                                exec,
                                src_type,
                                &icon_dst,
                                ImagePreprocess::ResizeAndCropSquare256,
                            )
                            .await?;
                        }

                        if let Some(gc) = egs_to_agc.get(&item.egs_id) {
                            if !gc.thumbnail_url.is_empty() {
                                let thumb_dst = resolver.thumbnail_png_path(collection_id.value);
                                iq.enqueue(
                                    &gc.thumbnail_url,
                                    ImageSrcType::Url,
                                    &thumb_dst,
                                    ImagePreprocess::ResizeForWidth400,
                                )
                                .await?;
                            }
                        }

                        processed += 1;
                    }

                    Ok::<usize, anyhow::Error>(processed)
                })
            })
            .await?;
        Ok(count)
    }

    pub(crate) async fn update_explored_cache(
        &self,
        explored_to_add: Vec<String>,
    ) -> anyhow::Result<()> {
        if explored_to_add.is_empty() {
            return Ok(());
        }
        let _ = self
            .manager
            .run(|repos| {
                Box::pin(async move {
                    use std::collections::HashSet;
                    let before = repos.explored_cache().get_all().await?;
                    let adding: HashSet<String> = explored_to_add
                        .into_iter()
                        .filter(|v| !before.contains(v))
                        .collect();
                    if !adding.is_empty() {
                        let _ = repos.explored_cache().add(adding).await?;
                    }
                    Ok::<(), anyhow::Error>(())
                })
            })
            .await;
        Ok(())
    }

    pub async fn backfill_thumbnail_sizes(&self) -> anyhow::Result<usize> {
        let resolver = self.resolver.clone();
        let updated = self
            .manager
            .run(|repos| {
                let resolver = resolver.clone();
                Box::pin(async move {
                    use domain::repository::collection::CollectionRepository as _;
                    let mut coll = repos.collection();
                    let ids = coll.get_null_thumbnail_size_element_ids().await?;
                    let mut updated: usize = 0;
                    if !ids.is_empty() {
                        for id in ids.into_iter() {
                            let path = resolver.thumbnail_png_path(id.value);
                            match image::image_dimensions(&path) {
                                Ok((w, h)) => {
                                    let _ = coll
                                        .upsert_collection_element_thumbnail_size(
                                            &id, w as i32, h as i32,
                                        )
                                        .await;
                                    updated += 1;
                                }
                                Err(_) => {}
                            }
                        }
                    }
                    Ok::<usize, anyhow::Error>(updated)
                })
            })
            .await?;
        Ok(updated)
    }

    pub(crate) async fn prepare_link_tasks(
        &self,
        deduped: &[ResolvedWork],
    ) -> anyhow::Result<Vec<WorkLinkTask>> {
        if deduped.is_empty() {
            return Ok(Vec::new());
        }

        let mut uniq_egs: Vec<i32> = Vec::new();
        let mut seen_egs: HashSet<i32> = HashSet::new();
        for it in deduped.iter() {
            if seen_egs.insert(it.egs_id) {
                uniq_egs.push(it.egs_id);
            }
        }

        let egs_to_work: HashMap<i32, Id<domain::works::Work>> = self
            .manager
            .run(|repos| {
                let uniq_egs = uniq_egs.clone();
                Box::pin(async move {
                    let mut coll = repos.collection();
                    let egs_ce_pairs = coll
                        .get_collection_ids_by_erogamescape_ids(&uniq_egs)
                        .await?;
                    if egs_ce_pairs.is_empty() {
                        return Ok::<HashMap<i32, Id<domain::works::Work>>, anyhow::Error>(
                            HashMap::new(),
                        );
                    }
                    let ce_ids: Vec<Id<CollectionElement>> =
                        egs_ce_pairs.iter().map(|(_e, ce)| ce.clone()).collect();
                    let ce_work_pairs = coll.get_work_ids_by_collection_ids(&ce_ids).await?;
                    let mut ce_to_work: HashMap<Id<CollectionElement>, Id<domain::works::Work>> =
                        HashMap::new();
                    for (ce, w) in ce_work_pairs.into_iter() {
                        ce_to_work.insert(ce, w);
                    }
                    let mut map: HashMap<i32, Id<domain::works::Work>> = HashMap::new();
                    for (egs, ce) in egs_ce_pairs.into_iter() {
                        if let Some(w) = ce_to_work.get(&ce) {
                            map.insert(egs, w.clone());
                        }
                    }
                    Ok::<HashMap<i32, Id<domain::works::Work>>, anyhow::Error>(map)
                })
            })
            .await?;

        if egs_to_work.is_empty() {
            return Ok(Vec::new());
        }

        let mut tasks: Vec<WorkLinkTask> = Vec::new();
        let mut seen_work: HashSet<i32> = HashSet::new();
        for item in deduped.iter() {
            if let Some(wid) = egs_to_work.get(&item.egs_id) {
                if !seen_work.insert(wid.value) {
                    continue;
                }
                tasks.push(WorkLinkTask {
                    work_id: wid.clone(),
                    kind: item.candidate.kind.clone(),
                    src: item.candidate.path.clone(),
                });
            }
        }

        Ok(tasks)
    }
}
