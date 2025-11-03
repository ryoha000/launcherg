#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use std::collections::HashSet;
    use std::{path::PathBuf, sync::Arc, sync::Mutex};

    use domain::pubsub::{PubSubEvent, PubSubService};
    use domain::scan::{
        CandidateKind, MockDuplicateResolver, MockFileSystem, MockMetadataExtractor, ResolvedWork,
        WorkCandidate, WorkCandidateOrResolvedWork,
    };
    use domain::service::work_linker::MockWorkLinker;
    use domain::service::work_registration::MockWorkRegistrationService;

    use crate::repositorymock::{TestRepositories, TestRepositoryManager};

    use crate::work_pipeline::WorkPipelineUseCase;

    #[derive(Clone, Default)]
    struct MockPubSub {
        events: Arc<Mutex<Vec<PubSubEvent>>>,
    }
    impl PubSubService for MockPubSub {
        fn notify(&self, event: PubSubEvent) -> Result<(), anyhow::Error> {
            self.events.lock().unwrap().push(event);
            Ok(())
        }
    }

    fn default_linker() -> Arc<MockWorkLinker> {
        let mut linker = MockWorkLinker::new();
        linker
            .expect_ensure_links()
            .times(0..)
            .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        Arc::new(linker)
    }

    fn default_registrar() -> Arc<MockWorkRegistrationService> {
        let mut registrar = MockWorkRegistrationService::new();
        registrar
            .expect_register()
            .times(0..)
            .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
        Arc::new(registrar)
    }

    // smoke
    #[tokio::test]
    async fn Start_空ストリームでもエラーなく完了する() {
        let pubsub = MockPubSub::default();
        let mut fs = MockFileSystem::new();
        fs.expect_walk_dir()
            .returning(|_, _| Ok(Box::new(Vec::<WorkCandidate>::new().into_iter())));
        let fs = Arc::new(fs);
        let mut extractor = MockMetadataExtractor::new();
        extractor
            .expect_enrich()
            .returning(|c| Ok(WorkCandidateOrResolvedWork::Candidate(c)));
        let extractor = Arc::new(extractor);
        let mut dedup = MockDuplicateResolver::new();
        dedup.expect_resolve().returning(|items| items);
        let dedup = Arc::new(dedup);

        let repos = TestRepositories::default();
        {
            let mut work = repos.work.lock().await;
            work.expect_list_work_ids_missing_thumbnail_size()
                .times(0..)
                .returning(|| Box::pin(async { Ok::<_, anyhow::Error>(Vec::new()) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));

        let linker = default_linker();
        let registrar = default_registrar();
        let uc: WorkPipelineUseCase<_, _, _, _, _, _, _, _> = WorkPipelineUseCase::new(
            manager,
            pubsub.clone(),
            fs,
            extractor,
            dedup,
            Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default()),
            linker,
            registrar,
        );
        let roots: Vec<PathBuf> = vec![];
        let _ = uc.start(roots, false).await.unwrap();
        // 何も検出されないのでイベントは0～数件（内部での副作用無し）
    }

    // enrich 1件ごとに candidate/resolved を送る
    #[tokio::test]
    async fn enrich結果_候補と解決を1件ずつ通知する() {
        let pubsub = MockPubSub::default();
        let fs = Arc::new(MockFileSystem::new());
        let mut extractor = MockMetadataExtractor::new();
        extractor.expect_enrich().returning(|c| {
            if c.path.to_string_lossy().contains("a.exe") {
                Ok(WorkCandidateOrResolvedWork::Candidate(c))
            } else {
                Ok(WorkCandidateOrResolvedWork::Resolved(ResolvedWork::new(
                    c,
                    "B".into(),
                    2,
                    0.1,
                )))
            }
        });
        let extractor = Arc::new(extractor);
        let mut d = MockDuplicateResolver::new();
        d.expect_resolve().returning(|items| items);
        let dedup = Arc::new(d);
        let manager = Arc::new(TestRepositoryManager::new(TestRepositories::default()));
        let linker = default_linker();
        let registrar = default_registrar();
        let uc: WorkPipelineUseCase<_, _, _, _, _, _, _, _> = WorkPipelineUseCase::new(
            manager,
            pubsub.clone(),
            fs,
            extractor,
            dedup,
            Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default()),
            linker,
            registrar,
        );
        let _ = uc
            .enrich_candidates_parallel_stream(futures::stream::iter(vec![
                WorkCandidate::new(PathBuf::from("a.exe"), CandidateKind::Exe),
                WorkCandidate::new(PathBuf::from("b.exe"), CandidateKind::Exe),
            ]))
            .await;
        let events = pubsub.events.lock().unwrap();
        assert!(events.iter().any(|event| matches!(
            event,
            PubSubEvent::ScanEnrichResult(payload)
                if payload.status == "candidate"
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            PubSubEvent::ScanEnrichResult(payload)
                if payload.status == "resolved"
        )));
    }

    // dedup は重複数のみ通知

    // open_candidate_stream
    #[tokio::test]
    async fn open_candidate_stream_キャッシュ無しでexclude_None() {
        let pubsub = MockPubSub::default();
        let mut fs = MockFileSystem::new();
        fs.expect_walk_dir()
            .withf(|_roots, exclude| exclude.is_none())
            .returning(|_, _| Ok(Box::new(Vec::<WorkCandidate>::new().into_iter())));
        let fs = Arc::new(fs);
        let extractor = Arc::new(MockMetadataExtractor::new());
        let dedup = Arc::new(MockDuplicateResolver::new());
        let manager = Arc::new(TestRepositoryManager::new(TestRepositories::default()));
        let linker = default_linker();
        let registrar = default_registrar();
        let uc: WorkPipelineUseCase<_, _, _, _, _, _, _, _> = WorkPipelineUseCase::new(
            manager,
            pubsub,
            fs,
            extractor,
            dedup,
            Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default()),
            linker,
            registrar,
        );
        let _ = uc.open_candidate_stream(&[], false).await.unwrap();
    }

    #[tokio::test]
    async fn open_candidate_stream_キャッシュ有りでexclude_Some() {
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
            explored
                .expect_get_all()
                .times(1)
                .returning(|| Box::pin(async move { Ok::<_, anyhow::Error>(HashSet::new()) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let linker = default_linker();
        let registrar = default_registrar();
        let uc: WorkPipelineUseCase<_, _, _, _, _, _, _, _> = WorkPipelineUseCase::new(
            manager,
            pubsub,
            fs,
            extractor,
            dedup,
            Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default()),
            linker,
            registrar,
        );
        let _ = uc.open_candidate_stream(&[], true).await.unwrap();
    }

    // deduplicate_and_notify
    #[tokio::test]
    async fn deduplicate_and_notify_重複数のみscanDedupで通知される() {
        let pubsub = MockPubSub::default();
        let fs = Arc::new(MockFileSystem::new());
        let extractor = Arc::new(MockMetadataExtractor::new());
        let mut d = MockDuplicateResolver::new();
        d.expect_resolve()
            .returning(|items| items.into_iter().take(1).collect());
        let dedup = Arc::new(d);
        let manager = Arc::new(TestRepositoryManager::new(TestRepositories::default()));
        let linker = default_linker();
        let registrar = default_registrar();
        let uc: WorkPipelineUseCase<_, _, _, _, _, _, _, _> = WorkPipelineUseCase::new(
            manager,
            pubsub.clone(),
            fs,
            extractor,
            dedup,
            Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default()),
            linker,
            registrar,
        );
        let resolved = vec![
            ResolvedWork::new(
                WorkCandidate::new(PathBuf::from("a.exe"), CandidateKind::Exe),
                "a".into(),
                1,
                0.1,
            ),
            ResolvedWork::new(
                WorkCandidate::new(PathBuf::from("b.exe"), CandidateKind::Exe),
                "b".into(),
                2,
                0.2,
            ),
        ];
        let (_deduped, dup) = uc.deduplicate_and_notify(resolved, 2);
        assert_eq!(dup, 1);
        let events = pubsub.events.lock().unwrap();
        assert!(events.iter().any(|event| matches!(
            event,
            PubSubEvent::ScanDedup(payload) if payload.removed_count == 1
        )));
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
            work.expect_find_by_title()
                .returning(|_| Box::pin(async { Ok(None) }));
            work.expect_find_work_ids_by_erogamescape_ids()
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(vec![]) }));
            work.expect_upsert()
                .returning(|_| Box::pin(async { Ok(domain::StrId::new("100".to_string())) }));
            work.expect_upsert_erogamescape_map()
                .returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        }
        {
            let mut agc = repos.all_game_cache.lock().await;
            agc.expect_get_by_ids()
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(vec![]) }));
        }
        {
            let mut iq = repos.image_queue.lock().await;
            iq.expect_enqueue().returning(|_, _, _, _| {
                Box::pin(async { Ok::<_, anyhow::Error>(domain::Id::new(1)) })
            });
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let linker = default_linker();
        let registrar = default_registrar();
        let uc: WorkPipelineUseCase<_, _, _, _, _, _, _, _> = WorkPipelineUseCase::new(
            manager,
            pubsub,
            fs,
            extractor,
            dedup,
            Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default()),
            linker,
            registrar,
        );

        let items = vec![ResolvedWork::new(
            WorkCandidate::new(PathBuf::from("C:/path/a.exe"), CandidateKind::Exe),
            "A".into(),
            10,
            0.1,
        )];
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
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let linker = default_linker();
        let mut registrar = MockWorkRegistrationService::new();
        registrar.expect_register().times(1).returning(|requests| {
            // WorkRegistrationRequestが正しく作成されていることを確認
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].keys.len(), 1);
            if let domain::service::work_registration::UniqueWorkKey::ErogamescapeId(id) =
                &requests[0].keys[0]
            {
                assert_eq!(*id, 10);
            } else {
                panic!("Expected ErogamescapeId");
            }
            assert_eq!(requests[0].insert.title, "A");
            assert!(requests[0].insert.path.is_some());
            // icon と thumbnail が設定されていることを確認
            assert!(requests[0].insert.icon.is_some());
            assert!(requests[0].insert.thumbnail.is_some());
            Box::pin(async {
                Ok::<_, anyhow::Error>(vec![
                    domain::service::work_registration::WorkRegistrationResult {
                        resolved_keys: vec![
                            domain::service::work_registration::UniqueWorkKey::ErogamescapeId(10),
                        ],
                        work_id: domain::StrId::new("100".to_string()),
                    },
                ])
            })
        });
        let registrar = Arc::new(registrar);
        let uc: WorkPipelineUseCase<_, _, _, _, _, _, _, _> = WorkPipelineUseCase::new(
            manager,
            pubsub,
            fs,
            extractor,
            dedup,
            Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default()),
            linker,
            registrar,
        );
        let items = vec![ResolvedWork::new(
            WorkCandidate::new(PathBuf::from("C:/path/a.exe"), CandidateKind::Exe),
            "A".into(),
            10,
            0.1,
        )];
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
            work.expect_find_by_title()
                .returning(|_| Box::pin(async { Ok(None) }));
            work.expect_find_work_ids_by_erogamescape_ids()
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(vec![]) }));
            work.expect_upsert()
                .returning(|_| Box::pin(async { Err(anyhow::anyhow!("fail")) }));
        }
        {
            let mut agc = repos.all_game_cache.lock().await;
            agc.expect_get_by_ids()
                .returning(|_| Box::pin(async { Ok::<_, anyhow::Error>(vec![]) }));
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let linker = default_linker();
        let mut registrar = MockWorkRegistrationService::new();
        registrar
            .expect_register()
            .times(1)
            .returning(|_| Box::pin(async { Err(anyhow::anyhow!("fail")) }));
        let registrar = Arc::new(registrar);
        let uc: WorkPipelineUseCase<_, _, _, _, _, _, _, _> = WorkPipelineUseCase::new(
            manager,
            pubsub.clone(),
            fs,
            extractor,
            dedup,
            Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default()),
            linker,
            registrar,
        );
        let items = vec![ResolvedWork::new(
            WorkCandidate::new(PathBuf::from("C:/path/a.exe"), CandidateKind::Exe),
            "A".into(),
            10,
            0.1,
        )];
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
            explored.expect_get_all().returning(|| {
                Box::pin(async {
                    Ok::<_, anyhow::Error>(HashSet::from_iter(vec!["a".to_string()]))
                })
            });
            explored.expect_add().returning(|adding| {
                assert!(adding.contains("b"));
                assert!(!adding.contains("a"));
                Box::pin(async { Ok::<_, anyhow::Error>(()) })
            });
        }
        let manager = Arc::new(TestRepositoryManager::new(repos));
        let linker = default_linker();
        let registrar = default_registrar();
        let uc: WorkPipelineUseCase<_, _, _, _, _, _, _, _> = WorkPipelineUseCase::new(
            manager,
            pubsub,
            fs,
            extractor,
            dedup,
            Arc::new(domain::service::save_path_resolver::DirsSavePathResolver::default()),
            linker,
            registrar,
        );
        let _ = uc
            .update_explored_cache(vec!["a".into(), "b".into()])
            .await
            .unwrap();
    }
}
