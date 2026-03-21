use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicI32, Ordering};
use std::{marker::PhantomData, sync::Arc};

use crate::work_thumbnail::WorkThumbnailUseCase;
use domain::pubsub::{DedupResultPayload, EnrichResultPayload, PubSubEvent, PubSubService};
use domain::repository::{
    explored_cache::ExploredCacheRepository as _, manager::RepositoryManager, RepositoriesExt,
};
use domain::scan::{
    CandidateKind, DuplicateResolver, FileSystem, MetadataExtractor, ResolvedWork, WorkCandidate,
    WorkCandidateOrResolvedWork,
};
use domain::service::save_path_resolver::SavePathResolver;
use domain::service::work_linker::{WorkLinkTask, WorkLinker};
use domain::service::work_registration::{
    ImageApply, ImageSource, ImageStrategy, RegisterWorkPath, UniqueWorkKey, WorkInsert,
    WorkRegistrationService,
};
use domain::StrId;
use futures::StreamExt as _;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

pub struct WorkPipelineUseCase<M, R, P, FS, ME, DR, WL, RS>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService,
    FS: FileSystem,
    ME: MetadataExtractor,
    DR: DuplicateResolver,
    WL: WorkLinker,
    RS: WorkRegistrationService + Send + Sync + 'static,
{
    manager: Arc<M>,
    pubsub: P,
    fs: Arc<FS>,
    extractor: Arc<ME>,
    dedup: Arc<DR>,
    resolver: Arc<dyn SavePathResolver>,
    linker: Arc<WL>,
    registrar: Arc<RS>,
    _marker: PhantomData<R>,
}

impl<M, R, P, FS, ME, DR, WL, RS> WorkPipelineUseCase<M, R, P, FS, ME, DR, WL, RS>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
    P: PubSubService,
    FS: FileSystem,
    ME: MetadataExtractor + Send + Sync + 'static,
    DR: DuplicateResolver,
    WL: WorkLinker + Send + Sync + 'static,
    RS: WorkRegistrationService + Send + Sync + 'static,
{
    pub fn new(
        manager: Arc<M>,
        pubsub: P,
        fs: Arc<FS>,
        extractor: Arc<ME>,
        dedup: Arc<DR>,
        resolver: Arc<dyn SavePathResolver>,
        linker: Arc<WL>,
        registrar: Arc<RS>,
    ) -> Self {
        Self {
            manager,
            pubsub,
            fs,
            extractor,
            dedup,
            resolver,
            linker,
            registrar,
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
        // ResolvedWork を WorkRegistrationRequest に変換
        let requests: Vec<domain::service::work_registration::WorkRegistrationRequest> = deduped
            .iter()
            .map(|item| {
                let item_file_path = item.candidate.path.to_string_lossy().to_string();
                let path = match item.candidate.kind {
                    CandidateKind::Exe => Some(RegisterWorkPath::Exe {
                        exe_path: item_file_path.clone(),
                    }),
                    CandidateKind::Shortcut => Some(RegisterWorkPath::Lnk {
                        lnk_path: item_file_path.clone(),
                    }),
                    CandidateKind::Folder | CandidateKind::Other => None,
                };

                // icon: FromPath/OnlyIfMissing
                let icon = path.as_ref().map(|p| ImageApply {
                    strategy: ImageStrategy::Always,
                    source: ImageSource::FromPath(p.clone()),
                });

                // thumbnail: FromEgs/OnlyIfMissing
                let thumbnail = Some(ImageApply {
                    strategy: ImageStrategy::OnlyIfMissing,
                    source: ImageSource::FromEgs,
                });

                domain::service::work_registration::WorkRegistrationRequest {
                    keys: vec![UniqueWorkKey::ErogamescapeId(item.egs_id)],
                    insert: WorkInsert {
                        title: item.title.clone(),
                        path,
                        egs_info: None,
                        icon,
                        thumbnail,
                        parent_pack_dmm_key: None,
                    },
                }
            })
            .collect();

        let _ = self.registrar.register(requests).await?;
        Ok(deduped.len())
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
        let work_thumbnail_use_case =
            WorkThumbnailUseCase::new(self.manager.clone(), self.resolver.clone());
        work_thumbnail_use_case.backfill_thumbnail_sizes().await
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

        let egs_to_work: HashMap<i32, StrId<domain::works::Work>> = self
            .manager
            .run(|_repos| {
                let _uniq_egs = uniq_egs.clone();
                Box::pin(async move {
                    // 旧 CE マップは廃止。EGS→Work は存在しない可能性があるため空
                    Ok::<HashMap<i32, StrId<domain::works::Work>>, anyhow::Error>(HashMap::new())
                })
            })
            .await?;

        if egs_to_work.is_empty() {
            return Ok(Vec::new());
        }

        let mut tasks: Vec<WorkLinkTask> = Vec::new();
        let mut seen_work: HashSet<String> = HashSet::new();
        for item in deduped.iter() {
            if let Some(wid) = egs_to_work.get(&item.egs_id) {
                if !seen_work.insert(wid.value.clone()) {
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
