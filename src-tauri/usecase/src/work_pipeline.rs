use std::{marker::PhantomData, sync::Arc, time::Instant};

use domain::pubsub::{PubSubService, ScanProgressPayload, ScanLogPayload, ScanSummaryPayload, ScanPhaseTimingPayload, ScanPipelineStatsPayload};
use domain::scan::{CandidateKind, DuplicateResolver, FileSystem, MetadataExtractor, ResolvedWork, ScanStats, WorkCandidate, WorkCandidateOrResolvedWork};
use domain::Id;
use domain::collection::CollectionElement;
use domain::repository::{
    RepositoriesExt,
    manager::RepositoryManager,
    all_game_cache::AllGameCacheRepository as _,
    collection::CollectionRepository as _,
    works::WorkRepository as _,
    explored_cache::ExploredCacheRepository as _,
    save_image_queue::ImageSaveQueueRepository as _,
};
use domain::save_image_queue::{ImageSrcType, ImagePreprocess};
use domain::service::save_path_resolver::{SavePathResolver, DirsSavePathResolver};
use futures::StreamExt as _;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct WorkPipelineUseCase<M, R, P, FS, ME, DR>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService,
    FS: FileSystem,
    ME: MetadataExtractor,
    DR: DuplicateResolver,
{
    manager: Arc<M>,
    pubsub: P,
    fs: Arc<FS>,
    extractor: Arc<ME>,
    dedup: Arc<DR>,
    _marker: PhantomData<R>,
}

impl<M, R, P, FS, ME, DR> WorkPipelineUseCase<M, R, P, FS, ME, DR>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService,
    FS: FileSystem,
    ME: MetadataExtractor + Send + Sync + 'static,
    DR: DuplicateResolver,
{
    pub fn new(manager: Arc<M>, pubsub: P, fs: Arc<FS>, extractor: Arc<ME>, dedup: Arc<DR>) -> Self { Self { manager, pubsub, fs, extractor, dedup, _marker: PhantomData } }

    pub async fn start(&self, roots: Vec<std::path::PathBuf>, use_cache: bool) -> anyhow::Result<()> {
        let started = Instant::now();
        let mut stats = ScanStats::new(0, 0, 0, 0, 0);
        let errors = 0i32;

        // フェーズ: ルート列挙
        self.notify_phase("EnumeratingRoots", 0, 0, 0, Some("ルート列挙"));

        // フェーズ: FS 走査（exclude は内部で解決）→ mpsc チャネルで enrich へストリーミング
        self.notify_phase("WalkingFs", 0, 0, 0, Some("候補抽出"));
        let t_build = Instant::now();
        let iter = self.build_candidate_iter(&roots, use_cache).await?;
        self.notify_phase_timing("BuildCandidateIter", t_build.elapsed());
        let (tx, rx) = mpsc::channel::<WorkCandidate>(2048);
        let t_walk = Instant::now();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel::<(i64, i64, i64)>();
        tokio::spawn(async move {
            let mut enumerated: i64 = 0;
            let mut produce_block_ms: i64 = 0;
            for c in iter {
                enumerated += 1;
                match tx.try_send(c) {
                    Ok(_) => {}
                    Err(tokio::sync::mpsc::error::TrySendError::Full(c)) => {
                        let t = Instant::now();
                        if tx.send(c).await.is_err() { break; }
                        produce_block_ms += t.elapsed().as_millis() as i64;
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => break,
                }
            }
            let walking_ms = t_walk.elapsed().as_millis() as i64;
            let _ = done_tx.send((walking_ms, enumerated, produce_block_ms));
        });
        self.notify_phase("Classifying", 0, 0, 0, Some("認識"));
        // フェーズ: メタ付与（並列・ストリーミング）
        let t_enrich = Instant::now();
        let (resolved, explored, processed_count) = self.enrich_candidates_parallel_stream(ReceiverStream::new(rx)).await;
        let enriching_ms = t_enrich.elapsed().as_millis() as i64;
        self.notify_phase_timing("Enriching", std::time::Duration::from_millis(enriching_ms as u64));
        if let Ok((walking_ms, enumerated_count, producer_block_ms)) = done_rx.await {
            self.notify_phase_timing("WalkingFs", std::time::Duration::from_millis(walking_ms as u64));
            let processed_count = processed_count as i64;
            let backlog_ms = (enriching_ms - walking_ms).max(0);
            let producer_rate_per_s = if walking_ms > 0 { enumerated_count as f64 / (walking_ms as f64 / 1000.0) } else { 0.0 };
            let consumer_rate_per_s = if enriching_ms > 0 { processed_count as f64 / (enriching_ms as f64 / 1000.0) } else { 0.0 };
            let _ = self.pubsub.notify(
                "scanPipelineStats",
                ScanPipelineStatsPayload::new(
                    enumerated_count,
                    processed_count,
                    walking_ms,
                    enriching_ms,
                    backlog_ms,
                    producer_block_ms,
                    producer_rate_per_s,
                    consumer_rate_per_s,
                ),
            );
        }
        stats.found = processed_count;
        stats.recognized = resolved.len();
        let total = stats.found as i32;

        // フェーズ: 重複排除
        let t_dedup = Instant::now();
        let (deduped, duplicates) = self.deduplicate_and_notify(resolved, stats.recognized);
        self.notify_phase_timing("Deduping", t_dedup.elapsed());
        stats.duplicates = duplicates;

        // フェーズ: 永続化
        let t_persist = Instant::now();
        let persisted = self.persist(&deduped).await?;
        self.notify_phase_timing("Persisting", t_persist.elapsed());
        let t_update_cache = Instant::now();
        let _ = self.update_explored_cache(explored).await;
        self.notify_phase_timing("UpdateExploredCache", t_update_cache.elapsed());
        stats.persisted = persisted;
        self.notify_phase("Persisting", persisted as i32, stats.recognized as i32, errors, Some("保存"));

        // フェーズ: PostProcessing（通知→実処理）
        self.notify_phase("PostProcessing", persisted as i32, total, errors, Some("事後処理"));
        let t_post = Instant::now();
        let _ = self.post_process_thumbnail_sizes().await;
        self.notify_phase_timing("PostProcessing", t_post.elapsed());

        // サマリ通知
        let took = started.elapsed().as_millis() as i64;
        self.notify_summary(took, &stats);
        Ok(())
    }

    fn notify_phase_timing(&self, phase: &str, took: std::time::Duration) {
        let _ = self.pubsub.notify(
            "scanPhaseTiming",
            ScanPhaseTimingPayload::new(phase.to_string(), took.as_millis() as i64),
        );
    }

    pub(crate) fn notify_phase(&self, phase: &str, current: i32, total: i32, errors: i32, label: Option<&str>) {
        let _ = self.pubsub.notify(
            "scanProgress",
            ScanProgressPayload::new(
                phase.into(),
                current,
                total,
                errors,
                label.map(|s| s.to_string()),
            ),
        );
    }

    pub(crate) fn notify_summary(&self, took_ms: i64, stats: &ScanStats) {
        let _ = self.pubsub.notify(
            "scanSummary",
            ScanSummaryPayload::new(
                took_ms,
                stats.found as i32,
                stats.recognized as i32,
                stats.persisted as i32,
                stats.skipped as i32,
                stats.duplicates as i32,
            ),
        );
        let _ = self.pubsub.notify(
            "progress",
            domain::pubsub::ProgressPayload::new(format!(
                "スキャン完了: {}ms, 件数 found={}, recognized={}, persisted={}, skipped={}, duplicates={}",
                took_ms, stats.found, stats.recognized, stats.persisted, stats.skipped, stats.duplicates
            )),
        );
        let _ = self.pubsub.notify("progresslive", domain::pubsub::ProgressLivePayload::new(None));
    }

    pub(crate) async fn build_candidate_iter(&self, roots: &[std::path::PathBuf], use_cache: bool) -> anyhow::Result<Box<dyn Iterator<Item = WorkCandidate> + Send>> {
        let exclude = if use_cache {
            let cache = self.manager.run(|repos| {
                Box::pin(async move { Ok::<_, anyhow::Error>(repos.explored_cache().get_all().await?) })
            }).await?;
            Some(std::sync::Arc::new(cache))
        } else { None };
        let iter = self.fs.walk_dir(roots, exclude)?;
        Ok(iter)
    }

    pub(crate) async fn enrich_candidates_parallel_stream<S>(&self, candidates: S) -> (Vec<ResolvedWork>, Vec<String>, usize)
    where
        S: futures::Stream<Item = WorkCandidate> + Unpin,
    {
        use std::sync::atomic::{AtomicI32, Ordering};
        let extractor = self.extractor.clone();
        let processed = std::sync::Arc::new(AtomicI32::new(0));
        let pubsub_en = &self.pubsub;
        let processed_counter = processed.clone();
        // 並列度は CPUスレッド×4（最大512）で固定
        let default_parallel = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
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
                    let current = processed.fetch_add(1, Ordering::Relaxed) + 1;
                    let _ = pubsub.notify(
                        "scanProgress",
                        ScanProgressPayload::new(
                            "Enriching".into(),
                            current,
                            0,
                            0,
                            Some("メタ付与中".into()),
                        ),
                    );
                    match res {
                        Ok(Ok(v)) => Some(v),
                        Ok(Err(e)) => {
                            let _ = pubsub.notify("scanLog", ScanLogPayload::new("error".into(), e.to_string()));
                            None
                        },
                        Err(join_err) => {
                            let _ = pubsub.notify("scanLog", ScanLogPayload::new("error".into(), format!("join error: {}", join_err)));
                            None
                        }
                    }
                }
            })
            .collect::<Vec<_>>()
            .await;
        let explored = enriched.iter().map(|v| match v {
            WorkCandidateOrResolvedWork::Candidate(c) => c.path.to_string_lossy().to_string(),
            WorkCandidateOrResolvedWork::Resolved(r) => r.candidate.path.to_string_lossy().to_string(),
        }).collect();
        let results = enriched.into_iter().filter_map(|v| match v {
            WorkCandidateOrResolvedWork::Candidate(_) => None,
            WorkCandidateOrResolvedWork::Resolved(r) => Some(r),
        }).collect();
        let count = processed.load(Ordering::Relaxed) as usize;
        (results, explored, count)
    }

    pub(crate) async fn enrich_candidates_parallel(&self, candidates: Box<dyn Iterator<Item = WorkCandidate> + Send>) -> (Vec<ResolvedWork>, Vec<String>, usize) {
        let stream = futures::stream::iter(candidates);
        self.enrich_candidates_parallel_stream(stream).await
    }

    pub(crate) fn deduplicate_and_notify(&self, resolved: Vec<ResolvedWork>, recognized_len: usize) -> (Vec<ResolvedWork>, usize) {
        let deduped: Vec<ResolvedWork> = self.dedup.resolve(resolved);
        let duplicates = recognized_len.saturating_sub(deduped.len());
        self.notify_phase(
            "Deduping",
            deduped.len() as i32,
            recognized_len as i32,
            0,
            Some("重複排除"),
        );
        (deduped, duplicates)
    }

    pub(crate) async fn persist(&self, deduped: &[ResolvedWork]) -> anyhow::Result<usize> {
        use std::collections::{HashMap, HashSet};

        // 単一トランザクションで全件処理
        let count = self.manager.run_in_transaction(|repos| {
            Box::pin(async move {
                // 先読みキャッシュを準備（EGS -> CE、CE -> Work）
                let mut coll = repos.collection();
                let mut work_repo = repos.work();
                let resolver = DirsSavePathResolver::default();

                let mut uniq_egs: Vec<i32> = Vec::new();
                let mut seen: HashSet<i32> = HashSet::new();
                for it in deduped.iter() {
                    if seen.insert(it.egs_id) { uniq_egs.push(it.egs_id); }
                }

                let mut egs_to_ce: HashMap<i32, Id<CollectionElement>> = HashMap::new();
                if !uniq_egs.is_empty() {
                    let egs_ce_pairs = coll.get_collection_ids_by_erogamescape_ids(&uniq_egs).await?; // Vec<(egs, ce)>
                    for (egs, ce) in egs_ce_pairs.into_iter() { egs_to_ce.insert(egs, ce); }
                }

                let mut ce_to_work: HashMap<Id<CollectionElement>, Id<domain::works::Work>> = HashMap::new();
                let ce_ids: Vec<Id<CollectionElement>> = egs_to_ce.values().cloned().collect();
                if !ce_ids.is_empty() {
                    let ce_work_pairs = coll.get_work_ids_by_collection_ids(&ce_ids).await?; // Vec<(ce, work)>
                    for (ce, w) in ce_work_pairs.into_iter() { ce_to_work.insert(ce, w); }
                }

                // 先読み: EGS -> AllGameCacheOneWithThumbnailUrl
                let mut egs_to_agc: HashMap<i32, domain::all_game_cache::AllGameCacheOneWithThumbnailUrl> = HashMap::new();
                if !uniq_egs.is_empty() {
                    let list = repos.all_game_cache().get_by_ids(uniq_egs.clone()).await?;
                    for gc in list.into_iter() { egs_to_agc.insert(gc.id, gc); }
                }

                let mut processed = 0usize;
                for item in deduped.iter() {
                    let title = item.title.clone();
                    let item_file_path = item.candidate.path.to_string_lossy().to_string();

                    // CollectionElement の存在確認（EGS マッピングで判定）
                    let collection_id: Id<CollectionElement> = if let Some(cid) = egs_to_ce.get(&item.egs_id).cloned() {
                        // 既存: 非 upsert の名称更新
                        coll.update_collection_element_gamename_by_id(&cid, &title).await?;
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
                        let work_id = work_repo.upsert(&domain::works::NewWork::new(title.clone())).await?;
                        coll.insert_work_mapping(&collection_id, work_id.clone()).await?;
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
                        iq.enqueue(exec, src_type, &icon_dst, ImagePreprocess::ResizeAndCropSquare256).await?;
                    }

                    if let Some(gc) = egs_to_agc.get(&item.egs_id) {
                        if !gc.thumbnail_url.is_empty() {
                            let thumb_dst = resolver.thumbnail_png_path(collection_id.value);
                            iq.enqueue(&gc.thumbnail_url, ImageSrcType::Url, &thumb_dst, ImagePreprocess::ResizeForWidth400).await?;
                        }
                    }

                    processed += 1;
                }

                Ok::<usize, anyhow::Error>(processed)
            })
        }).await?;
        Ok(count)
    }

    pub(crate) async fn update_explored_cache(&self, explored_to_add: Vec<String>) -> anyhow::Result<()> {
        if explored_to_add.is_empty() { return Ok(()); }
        let _ = self.manager.run(|repos| {
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
        }).await;
        Ok(())
    }

    pub(crate) async fn post_process_thumbnail_sizes(&self) -> anyhow::Result<()> {
        let _ = self.manager.run(|repos| {
            Box::pin(async move {
                use domain::repository::collection::CollectionRepository as _;
                let mut coll = repos.collection();
                let ids = coll.get_null_thumbnail_size_element_ids().await?;
                if !ids.is_empty() {
                    for id in ids.into_iter() {
                        let resolver = DirsSavePathResolver::default();
                        let path = resolver.thumbnail_png_path(id.value);
                        match image::image_dimensions(&path) {
                            Ok((w, h)) => { let _ = coll.upsert_collection_element_thumbnail_size(&id, w as i32, h as i32).await; }
                            Err(_) => {}
                        }
                    }
                }
                Ok::<(), anyhow::Error>(())
            })
        }).await;
        Ok(())
    }
}


