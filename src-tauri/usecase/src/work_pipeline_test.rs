#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use std::collections::{HashSet, HashMap};
    use std::{path::PathBuf, sync::Arc, sync::Mutex};

    use domain::pubsub::PubSubService;
    use domain::scan::{CandidateKind, ResolvedWork, WorkCandidate, WorkCandidateOrResolvedWork, MockFileSystem, MockMetadataExtractor, MockDuplicateResolver};
    
    use crate::repositorymock::{TestRepositories, TestRepositoryManager};

    use crate::work_pipeline::WorkPipelineUseCase;

    #[derive(Clone, Default)]
    struct MockPubSub { events: Arc<Mutex<Vec<(String, serde_json::Value)>>> }
    impl PubSubService for MockPubSub {
        fn notify<T: serde::Serialize + Clone>(&self, event: &str, payload: T) -> Result<(), anyhow::Error> {
            let val = serde_json::to_value(payload)?;
            self.events.lock().unwrap().push((event.to_string(), val));
            Ok(())
        }
    }

    // smoke: start sends initial EnumeratingRoots
    #[tokio::test]
    async fn IdleでStartを受けるとEnumeratingRootsへ遷移する() {
        let pubsub = MockPubSub::default();
        let mut fs = MockFileSystem::new();
        fs.expect_walk_dir().returning(|_, _| Ok(Box::new(Vec::<WorkCandidate>::new().into_iter())));
        let fs = Arc::new(fs);
        let mut extractor = MockMetadataExtractor::new();
        extractor.expect_enrich().returning(|c| Ok(WorkCandidateOrResolvedWork::Candidate(c)));
        let extractor = Arc::new(extractor);
        let mut dedup = MockDuplicateResolver::new();
        dedup.expect_resolve().returning(|items| items);
        let dedup = Arc::new(dedup);

        let repos = TestRepositories::default();
        {
            let mut coll = repos.collection.lock().await;
            coll.expect_get_collection_ids_by_erogamescape_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
            coll.expect_get_work_ids_by_collection_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
            coll.expect_get_null_thumbnail_size_element_ids().returning(|| {
                Box::pin(async move { Ok::<_, anyhow::Error>(vec![]) })
            });
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));

        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub.clone(), fs, extractor, dedup);
        let roots: Vec<PathBuf> = vec![];
        let _ = uc.start(roots, false).await;

        let events = pubsub.events.lock().unwrap().clone();
        let first = events.into_iter().next().expect("no events");
        assert_eq!(first.0, "scanProgress");
        assert_eq!(first.1.get("phase").unwrap().as_str().unwrap(), "EnumeratingRoots");
    }

    // notify_phase unit
    #[tokio::test]
    async fn notify_phase_がscanProgressを送る() {
        let pubsub = MockPubSub::default();
        let mut fs = MockFileSystem::new();
        fs.expect_walk_dir().returning(|_, _| Ok(Box::new(Vec::<WorkCandidate>::new().into_iter())));
        let fs = Arc::new(fs);
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let manager = Arc::new(TestRepositoryManager::new(TestRepositories::default()));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub.clone(), fs, extractor, dedup);
        uc.notify_phase("PhaseX", 1, 2, 3, Some("ラベル"));
        let events = pubsub.events.lock().unwrap();
        let last = events.last().unwrap();
        assert_eq!(last.0, "scanProgress");
        assert_eq!(last.1.get("phase").unwrap().as_str().unwrap(), "PhaseX");
        assert_eq!(last.1.get("processed").unwrap().as_i64().unwrap(), 1);
        assert_eq!(last.1.get("total").unwrap().as_i64().unwrap(), 2);
        assert_eq!(last.1.get("errors").unwrap().as_i64().unwrap(), 3);
        assert_eq!(last.1.get("message").unwrap().as_str().unwrap(), "ラベル");
    }

    // notify_summary unit
    #[tokio::test]
    async fn notify_summary_がsummaryとprogress系を送る() {
        let pubsub = MockPubSub::default();
        let fs = Arc::new(MockFileSystem::new());
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let manager = Arc::new(TestRepositoryManager::new(TestRepositories::default()));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub.clone(), fs, extractor, dedup);
        let stats = domain::scan::ScanStats::new(10, 7, 5, 1, 2);
        uc.notify_summary(1234, &stats);
        let events = pubsub.events.lock().unwrap().clone();
        let mut kinds: HashMap<String, i32> = HashMap::new();
        for (k, _) in events { *kinds.entry(k).or_default() += 1; }
        assert_eq!(kinds.get("scanSummary").copied().unwrap_or(0), 1);
        assert_eq!(kinds.get("progress").copied().unwrap_or(0), 1);
        assert_eq!(kinds.get("progresslive").copied().unwrap_or(0), 1);
    }

    // build_candidate_iter
    #[tokio::test]
    async fn build_candidate_iter_キャッシュ無しでexclude_None() {
        let pubsub = MockPubSub::default();
        let mut fs = MockFileSystem::new();
        fs.expect_walk_dir()
            .withf(|_roots, exclude| exclude.is_none())
            .returning(|_, _| Ok(Box::new(Vec::<WorkCandidate>::new().into_iter())));
        let fs = Arc::new(fs);
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let manager = Arc::new(TestRepositoryManager::new(TestRepositories::default()));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub, fs, extractor, dedup);
        let _ = uc.build_candidate_iter(&[], false).await.unwrap();
    }

    #[tokio::test]
    async fn build_candidate_iter_キャッシュ有りでexclude_Some() {
        let pubsub = MockPubSub::default();
        let mut fs = MockFileSystem::new();
        fs.expect_walk_dir()
            .withf(|_roots, exclude| exclude.is_some())
            .returning(|_, _| Ok(Box::new(Vec::<WorkCandidate>::new().into_iter())));
        let fs = Arc::new(fs);
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let repos = TestRepositories::default();
        {
            let mut explored = repos.explored_cache.lock().await;
            explored.expect_get_all().times(1).returning(|| Box::pin(async move { Ok::<_, anyhow::Error>(HashSet::new()) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub, fs, extractor, dedup);
        let _ = uc.build_candidate_iter(&[], true).await.unwrap();
    }

    // enrich_candidates_parallel
    #[tokio::test]
    async fn enrich_candidates_parallel_混在とエラーを正しく処理する() {
        let pubsub = MockPubSub::default();
        let mut extractor = MockMetadataExtractor::new();
        extractor.expect_enrich().returning(|c: WorkCandidate| {
            let name = c.path.to_string_lossy().to_string();
            if name.ends_with("ok.exe") {
                Ok(WorkCandidateOrResolvedWork::Resolved(ResolvedWork::new(c.clone(), "title".into(), 1, 0.1)))
            } else if name.ends_with("err.exe") {
                Err(anyhow::anyhow!("x"))
            } else {
                Ok(WorkCandidateOrResolvedWork::Candidate(c))
            }
        });
        let extractor = Arc::new(extractor);
        let fs = Arc::new(MockFileSystem::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let manager = Arc::new(TestRepositoryManager::new(TestRepositories::default()));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub.clone(), fs, extractor, dedup);

        let iter = vec![
            WorkCandidate::new(PathBuf::from("ok.exe"), CandidateKind::Exe),
            WorkCandidate::new(PathBuf::from("candidate.exe"), CandidateKind::Exe),
            WorkCandidate::new(PathBuf::from("err.exe"), CandidateKind::Exe),
        ].into_iter();
        let (resolved, explored, count) = uc.enrich_candidates_parallel(Box::new(iter)).await;
        assert_eq!(count, 3);
        assert_eq!(resolved.len(), 1);
        assert!(explored.iter().any(|p| p.ends_with("ok.exe")));
        assert!(explored.iter().any(|p| p.ends_with("candidate.exe")));

        let events = pubsub.events.lock().unwrap();
        assert!(events.iter().any(|(ev, v)| ev == "scanLog" && v.get("level").unwrap() == "error"));
    }

    // deduplicate_and_notify
    #[tokio::test]
    async fn deduplicate_and_notify_重複数が計算されscanProgressが出る() {
        let pubsub = MockPubSub::default();
        let fs = Arc::new(MockFileSystem::new());
        let extractor = Arc::new(MockMetadataExtractor::new());
        let mut d = MockDuplicateResolver::new();
        d.expect_resolve().returning(|items| items.into_iter().take(1).collect());
        let dedup = Arc::new(d);
        let manager = Arc::new(TestRepositoryManager::new(TestRepositories::default()));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub.clone(), fs, extractor, dedup);
        let resolved = vec![
            ResolvedWork::new(WorkCandidate::new(PathBuf::from("a.exe"), CandidateKind::Exe), "a".into(), 1, 0.1),
            ResolvedWork::new(WorkCandidate::new(PathBuf::from("b.exe"), CandidateKind::Exe), "b".into(), 2, 0.2),
        ];
        let (_deduped, dup) = uc.deduplicate_and_notify(resolved, 2);
        assert_eq!(dup, 1);
        let events = pubsub.events.lock().unwrap();
        assert!(events.iter().any(|(ev, v)| ev == "scanProgress" && v.get("phase").unwrap() == "Deduping"));
    }

    // persist
    #[tokio::test]
    async fn persist_アイコンのみ() {
        let pubsub = MockPubSub::default();
        let fs = Arc::new(MockFileSystem::new());
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let repos = TestRepositories::default();
        {
            let mut work = repos.work.lock().await;
            work.expect_upsert().returning(|_| Box::pin(async { Ok(domain::Id::new(100)) }));
        }
        {
            let mut coll = repos.collection.lock().await;
            coll.expect_get_collection_ids_by_erogamescape_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
            coll.expect_get_work_ids_by_collection_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
            coll.expect_allocate_new_collection_element_id().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(domain::Id::new(200)) }));
            coll.expect_upsert_erogamescape_map().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
            coll.expect_insert_work_mapping().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        }
        {
            let mut agc = repos.all_game_cache.lock().await;
            agc.expect_get_by_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(vec![]) }));
        }
        {
            let mut iq = repos.image_queue.lock().await;
            iq.expect_enqueue().returning(|_, _, _, _| Box::pin(async { Ok::<_, anyhow::Error>(domain::Id::new(1)) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub, fs, extractor, dedup);

        let items = vec![ResolvedWork::new(WorkCandidate::new(PathBuf::from("C:/path/a.exe"), CandidateKind::Exe), "A".into(), 10, 0.1)];
        let count = uc.persist(&items).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn persist_サムネも投入() {
        let pubsub = MockPubSub::default();
        let fs = Arc::new(MockFileSystem::new());
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let repos = TestRepositories::default();
        {
            let mut work = repos.work.lock().await;
            work.expect_upsert().returning(|_| Box::pin(async { Ok(domain::Id::new(100)) }));
        }
        {
            let mut coll = repos.collection.lock().await;
            coll.expect_get_collection_ids_by_erogamescape_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
            coll.expect_get_work_ids_by_collection_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
            coll.expect_allocate_new_collection_element_id().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(domain::Id::new(200)) }));
            coll.expect_upsert_erogamescape_map().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
            coll.expect_insert_work_mapping().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        }
        {
            let mut agc = repos.all_game_cache.lock().await;
            agc.expect_get_by_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(vec![domain::all_game_cache::AllGameCacheOneWithThumbnailUrl { id: 10, gamename: "A".into(), thumbnail_url: "http://example.com/a.png".into() }]) }));
        }
        {
            let mut iq = repos.image_queue.lock().await;
            iq.expect_enqueue().times(2).returning(|_, _, _, _| Box::pin(async { Ok::<_, anyhow::Error>(domain::Id::new(1)) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub, fs, extractor, dedup);
        let items = vec![ResolvedWork::new(WorkCandidate::new(PathBuf::from("C:/path/a.exe"), CandidateKind::Exe), "A".into(), 10, 0.1)];
        let count = uc.persist(&items).await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn persist_トランザクション内で失敗するとErrを返す() {
        let pubsub = MockPubSub::default();
        let fs = Arc::new(MockFileSystem::new());
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let repos = TestRepositories::default();
        {
            let mut work = repos.work.lock().await;
            work.expect_upsert().returning(|_| Box::pin(async { Err(anyhow::anyhow!("fail")) }));
        }
        {
            let mut coll = repos.collection.lock().await;
            coll.expect_get_collection_ids_by_erogamescape_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
            coll.expect_get_work_ids_by_collection_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
            coll.expect_allocate_new_collection_element_id().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(domain::Id::new(200)) }));
            coll.expect_upsert_erogamescape_map().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        }
        {
            let mut agc = repos.all_game_cache.lock().await;
            agc.expect_get_by_ids().returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(vec![]) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub.clone(), fs, extractor, dedup);
        let items = vec![ResolvedWork::new(WorkCandidate::new(PathBuf::from("C:/path/a.exe"), CandidateKind::Exe), "A".into(), 10, 0.1)];
        let res = uc.persist(&items).await;
        assert!(res.is_err());
    }

    // update_explored_cache
    #[tokio::test]
    async fn update_explored_cache_差分のみaddされる() {
        let pubsub = MockPubSub::default();
        let fs = Arc::new(MockFileSystem::new());
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let repos = TestRepositories::default();
        {
            use std::iter::FromIterator;
            let mut explored = repos.explored_cache.lock().await;
            explored.expect_get_all().returning(|| Box::pin(async { Ok::<_, anyhow::Error>(HashSet::from_iter(vec!["a".to_string()])) }));
            explored.expect_add().returning(|adding| {
                assert!(adding.contains("b"));
                assert!(!adding.contains("a"));
                Box::pin(async { Ok::<_, anyhow::Error>(()) })
            });
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub, fs, extractor, dedup);
        let _ = uc.update_explored_cache(vec!["a".into(), "b".into()]).await.unwrap();
    }

    // post_process_thumbnail_sizes (empty)
    #[tokio::test]
    async fn post_process_thumbnail_sizes_空なら何もしない() {
        let pubsub = MockPubSub::default();
        let fs = Arc::new(MockFileSystem::new());
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let repos = TestRepositories::default();
        {
            let mut coll = repos.collection.lock().await;
            coll.expect_get_null_thumbnail_size_element_ids().returning(|| Box::pin(async { Ok::<_, anyhow::Error>(vec![]) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let uc: WorkPipelineUseCase<_, _, _, _, _, _> = WorkPipelineUseCase::new(manager, pubsub, fs, extractor, dedup);
        let _ = uc.post_process_thumbnail_sizes().await.unwrap();
    }
}


