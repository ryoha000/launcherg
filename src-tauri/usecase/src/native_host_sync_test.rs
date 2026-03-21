#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use std::sync::Arc;

    use domain::service::work_registration::{
        MockWorkRegistrationService, UniqueWorkKey, WorkRegistrationRequest,
        WorkRegistrationResult,
    };

    use crate::native_host_sync::{
        DlsiteSyncGameParam, DmmPackKey, DmmSyncGameParam, EgsInfo, NativeHostSyncUseCase,
    };
    use crate::repositorymock::{TestRepositories, TestRepositoryManager};

    fn create_usecase<
        R: domain::service::work_registration::WorkRegistrationService + Send + Sync + 'static,
    >(
        manager: Arc<TestRepositoryManager>,
        registrar: Arc<R>,
    ) -> NativeHostSyncUseCase<TestRepositoryManager, TestRepositories, R> {
        NativeHostSyncUseCase::new(manager, registrar)
    }

    #[tokio::test]
    async fn sync_dmm_games_親パックキーを保持したまま一回で登録できる() {
        let repos = TestRepositories::default();
        let manager = Arc::new(TestRepositoryManager::new(repos));

        let mut mock_registrar = MockWorkRegistrationService::new();
        mock_registrar
            .expect_register()
            .times(1)
            .returning(|requests: Vec<WorkRegistrationRequest>| {
                assert_eq!(requests.len(), 1);
                let resolved_keys = requests[0].keys.clone();
                let parent_pack = requests[0].insert.parent_pack_dmm_key.clone();
                assert!(matches!(resolved_keys[0], UniqueWorkKey::Dmm { .. }));
                assert_eq!(
                    parent_pack,
                    Some(domain::work_parent_pack::ParentPackKey {
                        store_id: "pack-store".into(),
                        category: "package".into(),
                        subcategory: "bundle".into(),
                    })
                );
                Box::pin(async move {
                    Ok(vec![WorkRegistrationResult {
                        resolved_keys,
                        work_id: domain::StrId::new("work-1".into()),
                    }])
                })
            });

        let usecase = create_usecase(manager, Arc::new(mock_registrar));
        let result = usecase
            .sync_dmm_games(vec![DmmSyncGameParam {
                store_id: "sid".into(),
                category: "game".into(),
                subcategory: "pc".into(),
                gamename: "Game".into(),
                egs: Some(EgsInfo {
                    erogamescape_id: 1,
                    gamename: "Game".into(),
                    gamename_ruby: "Game".into(),
                    brandname: "Brand".into(),
                    brandname_ruby: "Brand".into(),
                    sellday: "2024-01-01".into(),
                    is_nukige: false,
                }),
                image_url: String::new(),
                parent_pack: Some(DmmPackKey {
                    store_id: "pack-store".into(),
                    category: "package".into(),
                    subcategory: "bundle".into(),
                }),
            }])
            .await
            .unwrap();

        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn sync_dlsite_games_一回で登録できる() {
        let repos = TestRepositories::default();
        let manager = Arc::new(TestRepositoryManager::new(repos));

        let mut mock_registrar = MockWorkRegistrationService::new();
        mock_registrar
            .expect_register()
            .times(1)
            .returning(|requests: Vec<WorkRegistrationRequest>| {
                assert_eq!(requests.len(), 1);
                let resolved_keys = requests[0].keys.clone();
                assert!(matches!(resolved_keys[0], UniqueWorkKey::Dlsite { .. }));
                assert_eq!(requests[0].insert.parent_pack_dmm_key, None);
                Box::pin(async move {
                    Ok(vec![WorkRegistrationResult {
                        resolved_keys,
                        work_id: domain::StrId::new("work-1".into()),
                    }])
                })
            });

        let usecase = create_usecase(manager, Arc::new(mock_registrar));
        let result = usecase
            .sync_dlsite_games(vec![DlsiteSyncGameParam {
                store_id: "rj".into(),
                category: "game".into(),
                gamename: "Dlsite Game".into(),
                egs: None,
                image_url: String::new(),
            }])
            .await
            .unwrap();

        assert_eq!(result, 1);
    }
}
