use std::{marker::PhantomData, sync::Arc, time::Instant};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI32, Ordering};

use domain::pubsub::{PubSubService, ScanProgressPayload, ScanLogPayload, ScanSummaryPayload};
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
use domain::windows::WindowsExt;
use domain::repository::work_lnk::{WorkLnkRepository as _, NewWorkLnk};
use domain::windows::shell_link::{CreateShortcutRequest, ShellLink as _};
use futures::StreamExt as _;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct WorkPipelineUseCase<M, R, P, FS, ME, DR, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService,
    FS: FileSystem,
    ME: MetadataExtractor,
    DR: DuplicateResolver,
    W: WindowsExt,
{
    manager: Arc<M>,
    pubsub: P,
    fs: Arc<FS>,
    extractor: Arc<ME>,
    dedup: Arc<DR>,
    resolver: Arc<dyn SavePathResolver>,
    windows: Arc<W>,
    _marker: PhantomData<R>,
}

impl<M, R, P, FS, ME, DR, W> WorkPipelineUseCase<M, R, P, FS, ME, DR, W>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService,
    FS: FileSystem,
    ME: MetadataExtractor + Send + Sync + 'static,
    DR: DuplicateResolver,
    W: WindowsExt + Send + Sync + 'static,
{
    pub fn new(
        manager: Arc<M>,
        pubsub: P,
        fs: Arc<FS>,
        extractor: Arc<ME>,
        dedup: Arc<DR>,
        resolver: Arc<dyn SavePathResolver>,
        windows: Arc<W>,
    ) -> Self { Self { manager, pubsub, fs, extractor, dedup, resolver, windows, _marker: PhantomData } }

    pub async fn start(&self, roots: Vec<std::path::PathBuf>, use_cache: bool) -> anyhow::Result<()> {
        let started = Instant::now();
        let mut stats = ScanStats::new(0, 0, 0, 0, 0);

        let rx = self.open_candidate_stream(&roots, use_cache).await?;
        // フェーズ: メタ付与（並列・ストリーミング）
        let (resolved, explored, processed_count) = self.enrich_candidates_parallel_stream(ReceiverStream::new(rx)).await;
        stats.found = processed_count;
        stats.recognized = resolved.len();

        // フェーズ: 重複排除
        let (deduped, duplicates) = self.deduplicate_and_notify(resolved, stats.recognized);
        stats.duplicates = duplicates;

        // フェーズ: 永続化
        let persisted = self.persist(&deduped).await?;
        // フェーズ: .lnk 保全
        self.ensure_work_lnks(&deduped).await?;
        self.update_explored_cache(explored).await?;
        stats.persisted = persisted;

        self.post_process_thumbnail_sizes().await?;

        // サマリ通知
        let took = started.elapsed().as_millis() as i64;
        self.notify_summary(took, &stats);
        Ok(())
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

    pub(crate) async fn open_candidate_stream(&self, roots: &[std::path::PathBuf], use_cache: bool) -> anyhow::Result<mpsc::Receiver<WorkCandidate>> {
        let exclude = if use_cache {
            let cache = self.manager.run(|repos| {
                Box::pin(async move { Ok::<_, anyhow::Error>(repos.explored_cache().get_all().await?) })
            }).await?;
            Some(std::sync::Arc::new(cache))
        } else { None };

        let iter = self.fs.walk_dir(roots, exclude)?;

        let (tx, rx) = mpsc::channel::<WorkCandidate>(2048);
        tokio::spawn(async move {
            for c in iter {
                match tx.try_send(c) {
                    Ok(_) => {}
                    Err(tokio::sync::mpsc::error::TrySendError::Full(c)) => {
                        if tx.send(c).await.is_err() { break; }
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => break,
                }
            }
        });
        Ok(rx)
    }

    pub(crate) async fn enrich_candidates_parallel_stream<S>(&self, candidates: S) -> (Vec<ResolvedWork>, Vec<String>, usize)
    where
        S: futures::Stream<Item = WorkCandidate> + Unpin,
    {
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
        self.manager.run(|repos| {
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
        }).await?;
        Ok(())
    }

    // DB への保存後に、必要であれば .lnk を作成/コピーし work_lnks へ登録する
    pub(crate) async fn ensure_work_lnks(&self, deduped: &[ResolvedWork]) -> anyhow::Result<()> {
        if deduped.is_empty() { return Ok(()); }

        // 1) bulk で EGS -> CE -> Work を解決
        let mut uniq_egs: Vec<i32> = Vec::new();
        let mut seen_egs: HashSet<i32> = HashSet::new();
        for it in deduped.iter() { if seen_egs.insert(it.egs_id) { uniq_egs.push(it.egs_id); } }

        let egs_to_work: HashMap<i32, Id<domain::works::Work>> = self.manager.run(|repos| {
            let uniq_egs = uniq_egs.clone();
            Box::pin(async move {
                let mut coll = repos.collection();
                // EGS -> CE
                let egs_ce_pairs = coll.get_collection_ids_by_erogamescape_ids(&uniq_egs).await?; // Vec<(egs, ce)>
                if egs_ce_pairs.is_empty() { return Ok::<HashMap<i32, Id<domain::works::Work>>, anyhow::Error>(HashMap::new()); }
                // CE -> Work
                let ce_ids: Vec<Id<CollectionElement>> = egs_ce_pairs.iter().map(|(_e, ce)| ce.clone()).collect();
                let ce_work_pairs = coll.get_work_ids_by_collection_ids(&ce_ids).await?; // Vec<(ce, work)>
                let mut ce_to_work: HashMap<Id<CollectionElement>, Id<domain::works::Work>> = HashMap::new();
                for (ce, w) in ce_work_pairs.into_iter() { ce_to_work.insert(ce, w); }
                let mut map: HashMap<i32, Id<domain::works::Work>> = HashMap::new();
                for (egs, ce) in egs_ce_pairs.into_iter() { if let Some(w) = ce_to_work.get(&ce) { map.insert(egs, w.clone()); } }
                Ok::<HashMap<i32, Id<domain::works::Work>>, anyhow::Error>(map)
            })
        }).await?;

        if egs_to_work.is_empty() { return Ok(()); }

        // 2) 処理対象の task を作成（work_id ごとに一意）
        struct Task { work_id: Id<domain::works::Work>, kind: CandidateKind, src: String, dst: String }
        let mut tasks: Vec<Task> = Vec::new();
        let mut seen_work: HashSet<i32> = HashSet::new();
        for item in deduped.iter() {
            if let Some(wid) = egs_to_work.get(&item.egs_id) {
                if !seen_work.insert(wid.value) { continue; }
                let src = item.candidate.path.to_string_lossy().to_string();
                let dst = self.resolver.lnk_new_path(wid.value);
                tasks.push(Task { work_id: wid.clone(), kind: item.candidate.kind.clone(), src, dst });
            }
        }
        if tasks.is_empty() { return Ok(()); }

        // 3) 既に work_lnks が存在する work はスキップ（単発 API をまとめて 1 run 内で呼ぶ）
        let tasks: Vec<Task> = self.manager.run(|repos| {
            Box::pin(async move {
                let mut repo = repos.work_lnk();
                let mut filtered: Vec<Task> = Vec::new();
                for t in tasks.into_iter() {
                    let existed = repo.list_by_work_id(t.work_id.clone()).await?;
                    if existed.is_empty() { filtered.push(t); }
                }
                Ok::<Vec<Task>, anyhow::Error>(filtered)
            })
        }).await?;
        if tasks.is_empty() { return Ok(()); }

        // 4) OS 操作: Shortcut は個別コピー、Exe は create_bulk を一度だけ
        let mut exe_reqs: Vec<CreateShortcutRequest> = Vec::new();
        let mut to_insert: Vec<(Id<domain::works::Work>, String)> = Vec::new();

        for t in tasks.iter() {
            let parent = std::path::Path::new(&t.dst).parent().map(|p| p.to_path_buf());
            if let Some(p) = parent { let _ = std::fs::create_dir_all(p); }
            if matches!(t.kind, CandidateKind::Shortcut) {
                if std::fs::copy(&t.src, &t.dst).is_ok() {
                    to_insert.push((t.work_id.clone(), t.dst.clone()));
                } else {
                    let _ = self.pubsub.notify("scanLog", ScanLogPayload::new("error".into(), format!("ensure_work_lnks copy failed: {} -> {}", t.src, t.dst)));
                }
            } else if matches!(t.kind, CandidateKind::Exe) {
                exe_reqs.push(CreateShortcutRequest { target_path: t.src.clone(), dest_lnk_path: t.dst.clone(), working_dir: None, arguments: None, icon_path: None });
            }
        }
        if !exe_reqs.is_empty() {
            self.windows.shell_link().create_bulk(exe_reqs)?;
            for t in tasks.iter().filter(|t| matches!(t.kind, CandidateKind::Exe)) {
                to_insert.push((t.work_id.clone(), t.dst.clone()));
            }
        }

        if to_insert.is_empty() { return Ok(()); }

        // 5) DB 書き込みはトランザクションで一括
        let _ = self.manager.run_in_transaction(|repos| {
            let to_insert = to_insert.clone();
            Box::pin(async move {
                let mut repo = repos.work_lnk();
                for (wid, lnk_path) in to_insert.into_iter() {
                    let _ = repo.insert(&NewWorkLnk { work_id: wid, lnk_path }).await?;
                }
                Ok::<(), anyhow::Error>(())
            })
        }).await?;

        Ok(())
    }
}


