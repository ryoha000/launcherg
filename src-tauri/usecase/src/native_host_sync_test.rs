#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use std::sync::Arc;

    use domain::service::work_registration::{
        ImageSource, ImageStrategy, MockWorkRegistrationService, UniqueWorkKey,
        WorkRegistrationRequest,
    };
    use domain::StrId;

    use crate::native_host_sync::{DlsiteSyncGameParam, DmmSyncGameParam, NativeHostSyncUseCase};
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
    async fn sync_dmm_games_omit対象はフィルタされる() {
        let repos = TestRepositories::default();
        let manager = Arc::new(TestRepositoryManager::new(repos.clone()));

        let omitted_work_id = StrId::new("omitted-work".to_string());

        // work_omit().list() が omitted_work_id を返すように設定
        {
            let work_id_for_omit = omitted_work_id.clone();
            let mut mock_omit = repos.work_omit.lock().await;
            mock_omit.expect_list().times(1).returning(move || {
                let work_id = work_id_for_omit.clone();
                Box::pin(async move {
                    Ok(vec![domain::work_omit::WorkOmit {
                        id: domain::Id::new(1),
                        work_id,
                    }])
                })
            });
        }

        // dmm_work().find_by_store_key() が omitted_work_id を返すように設定
        {
            let work_id_for_dmm = omitted_work_id.clone();
            let mut mock_dmm = repos.dmm_work.lock().await;
            mock_dmm
                .expect_find_by_store_key()
                .times(1)
                .withf(|store_id: &str, category: &str, subcategory: &str| {
                    store_id == "SID1" && category == "game" && subcategory == "pc"
                })
                .returning(move |_, _, _| {
                    let work_id = work_id_for_dmm.clone();
                    Box::pin(async move {
                        Ok(Some(domain::works::DmmWork {
                            id: domain::Id::new(1),
                            work_id,
                            store_id: "SID1".to_string(),
                            category: "game".to_string(),
                            subcategory: "pc".to_string(),
                            parent_pack: None,
                        }))
                    })
                });
        }

        // WorkRegistrationService は呼ばれない（omit フィルタで除外されるため）
        let mut mock_registrar = MockWorkRegistrationService::new();
        mock_registrar
            .expect_register()
            .times(0)
            .returning(|_| Box::pin(async move { Ok(Vec::new()) }));
        let registrar = Arc::new(mock_registrar);

        let usecase = create_usecase(manager, registrar);

        let params = vec![DmmSyncGameParam {
            store_id: "SID1".to_string(),
            category: "game".to_string(),
            subcategory: "pc".to_string(),
            gamename: "Omitted Game".to_string(),
            egs: None,
            image_url: String::new(),
            parent_pack: None,
        }];

        let result = usecase.sync_dmm_games(params).await.unwrap();
        // omit 対象なので登録されない（0件）
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn sync_dlsite_games_omit対象はフィルタされる() {
        let repos = TestRepositories::default();
        let manager = Arc::new(TestRepositoryManager::new(repos.clone()));

        let omitted_work_id = StrId::new("omitted-work".to_string());

        // work_omit().list() が omitted_work_id を返すように設定
        {
            let work_id_for_omit = omitted_work_id.clone();
            let mut mock_omit = repos.work_omit.lock().await;
            mock_omit.expect_list().times(1).returning(move || {
                let work_id = work_id_for_omit.clone();
                Box::pin(async move {
                    Ok(vec![domain::work_omit::WorkOmit {
                        id: domain::Id::new(1),
                        work_id,
                    }])
                })
            });
        }

        // dlsite_work().find_by_store_key() が omitted_work_id を返すように設定
        {
            let work_id_for_dlsite = omitted_work_id.clone();
            let mut mock_dlsite = repos.dlsite_work.lock().await;
            mock_dlsite
                .expect_find_by_store_key()
                .times(1)
                .withf(|store_id: &str, category: &str| store_id == "RJ1" && category == "game")
                .returning(move |_, _| {
                    let work_id = work_id_for_dlsite.clone();
                    Box::pin(async move {
                        Ok(Some(domain::works::DlsiteWork {
                            id: domain::Id::new(1),
                            work_id,
                            store_id: "RJ1".to_string(),
                            category: "game".to_string(),
                        }))
                    })
                });
        }

        // WorkRegistrationService は空のリクエストで呼ばれる（omit フィルタで除外されるため）
        let mut mock_registrar = MockWorkRegistrationService::new();
        mock_registrar
            .expect_register()
            .times(1)
            .withf(|requests: &Vec<WorkRegistrationRequest>| requests.is_empty())
            .returning(|_| Box::pin(async move { Ok(Vec::new()) }));
        let registrar = Arc::new(mock_registrar);

        let usecase = create_usecase(manager, registrar);

        let params = vec![DlsiteSyncGameParam {
            store_id: "RJ1".to_string(),
            category: "game".to_string(),
            gamename: "Omitted Game".to_string(),
            egs: None,
            image_url: String::new(),
        }];

        let result = usecase.sync_dlsite_games(params).await.unwrap();
        // omit 対象なので登録されない（0件）
        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn sync_dmm_games_omit対象でない場合は正常に登録される() {
        let repos = TestRepositories::default();
        let manager = Arc::new(TestRepositoryManager::new(repos.clone()));

        let normal_work_id = StrId::new("normal-work".to_string());

        // work_omit().list() が空を返すように設定
        {
            let mut mock_omit = repos.work_omit.lock().await;
            mock_omit
                .expect_list()
                .times(1)
                .returning(|| Box::pin(async move { Ok(Vec::new()) }));
        }

        // dmm_work().find_by_store_key() が normal_work_id を返すように設定
        {
            let work_id_for_dmm = normal_work_id.clone();
            let mut mock_dmm = repos.dmm_work.lock().await;
            mock_dmm
                .expect_find_by_store_key()
                .times(1)
                .withf(|store_id: &str, category: &str, subcategory: &str| {
                    store_id == "SID2" && category == "game" && subcategory == "pc"
                })
                .returning(move |_, _, _| {
                    let work_id = work_id_for_dmm.clone();
                    Box::pin(async move {
                        Ok(Some(domain::works::DmmWork {
                            id: domain::Id::new(2),
                            work_id,
                            store_id: "SID2".to_string(),
                            category: "game".to_string(),
                            subcategory: "pc".to_string(),
                            parent_pack: None,
                        }))
                    })
                });
        }

        // WorkRegistrationService が呼ばれる（omit 対象でない）
        let mut mock_registrar = MockWorkRegistrationService::new();
        mock_registrar
            .expect_register()
            .times(1)
            .withf(|requests: &Vec<WorkRegistrationRequest>| {
                requests.len() == 1
                    && requests[0].keys.len() == 1
                    && matches!(&requests[0].keys[0], UniqueWorkKey::Dmm { store_id, category, subcategory } 
                        if store_id == "SID2" && category == "game" && subcategory == "pc")
                    && requests[0].insert.title == "Normal Game"
            })
            .returning(|requests| {
                Box::pin(async move {
                    Ok(requests
                        .iter()
                        .map(|req| domain::service::work_registration::WorkRegistrationResult {
                            resolved_keys: req.keys.clone(),
                            work_id: StrId::new("normal-work".to_string()),
                        })
                        .collect())
                })
            });
        let registrar = Arc::new(mock_registrar);

        let usecase = create_usecase(manager, registrar);

        let params = vec![DmmSyncGameParam {
            store_id: "SID2".to_string(),
            category: "game".to_string(),
            subcategory: "pc".to_string(),
            gamename: "Normal Game".to_string(),
            egs: None,
            image_url: String::new(),
            parent_pack: None,
        }];

        let result = usecase.sync_dmm_games(params).await.unwrap();
        // 登録される（1件）
        assert_eq!(result, 1);
    }

    #[tokio::test]
    async fn sync_dmm_games_画像URLが設定される() {
        let repos = TestRepositories::default();
        let manager = Arc::new(TestRepositoryManager::new(repos.clone()));

        // work_omit().list() が空を返すように設定
        {
            let mut mock_omit = repos.work_omit.lock().await;
            mock_omit
                .expect_list()
                .times(1)
                .returning(|| Box::pin(async move { Ok(Vec::new()) }));
        }

        // dmm_work().find_by_store_key() が None を返すように設定（新規作成）
        {
            let mut mock_dmm = repos.dmm_work.lock().await;
            mock_dmm
                .expect_find_by_store_key()
                .times(1)
                .returning(|_, _, _| Box::pin(async move { Ok(None) }));
        }

        // WorkRegistrationService が呼ばれ、画像URLが正しく設定されていることを確認
        let mut mock_registrar = MockWorkRegistrationService::new();
        mock_registrar
            .expect_register()
            .times(1)
            .returning(|requests| {
                // アサーション: 画像URLが正しく設定されていることを確認
                assert_eq!(requests.len(), 1);
                let req = &requests[0];
                assert!(req.insert.icon.is_some());
                assert!(req.insert.thumbnail.is_some());
                match &req.insert.icon.as_ref().unwrap().source {
                    ImageSource::FromUrl(url) => {
                        assert_eq!(url, "https://pics.dmm.co.jp/image_ps.jpg")
                    }
                    _ => panic!("Icon source should be FromUrl"),
                }
                match &req.insert.thumbnail.as_ref().unwrap().source {
                    ImageSource::FromUrl(url) => {
                        // normalize_thumbnail_url により ps.jpg が pl.jpg に変換される
                        assert_eq!(
                            url, "https://pics.dmm.co.jp/image_pl.jpg",
                            "Thumbnail URL should be normalized from ps.jpg to pl.jpg"
                        );
                    }
                    _ => panic!("Thumbnail source should be FromUrl"),
                }
                assert_eq!(
                    req.insert.icon.as_ref().unwrap().strategy,
                    ImageStrategy::OnlyIfNew
                );
                assert_eq!(
                    req.insert.thumbnail.as_ref().unwrap().strategy,
                    ImageStrategy::OnlyIfNew
                );

                Box::pin(async move {
                    Ok(requests
                        .iter()
                        .map(
                            |req| domain::service::work_registration::WorkRegistrationResult {
                                resolved_keys: req.keys.clone(),
                                work_id: StrId::new("new-work".to_string()),
                            },
                        )
                        .collect())
                })
            });
        let registrar = Arc::new(mock_registrar);

        let usecase = create_usecase(manager, registrar);

        let params = vec![DmmSyncGameParam {
            store_id: "SID3".to_string(),
            category: "game".to_string(),
            subcategory: "pc".to_string(),
            gamename: "New Game".to_string(),
            egs: None,
            image_url: "https://pics.dmm.co.jp/image_ps.jpg".to_string(),
            parent_pack: None,
        }];

        let result = usecase.sync_dmm_games(params).await.unwrap();
        assert_eq!(result, 1);
    }
}
