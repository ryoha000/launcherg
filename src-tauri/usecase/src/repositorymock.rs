#[cfg(test)]
mockall::mock! {
    pub RepositoriesExtMock {}

    impl domain::repositoryv2::RepositoriesExt for RepositoriesExtMock {
        type CollectionRepo = domain::repositoryv2::collection::MockCollectionRepository;
        type ExploredCacheRepo = domain::repositoryv2::explored_cache::MockExploredCacheRepository;
        type AllGameCacheRepo = domain::repositoryv2::all_game_cache::MockAllGameCacheRepository;
        type ImageQueueRepo = domain::repositoryv2::save_image_queue::MockImageSaveQueueRepository;
        type HostLogRepo = domain::repositoryv2::native_host_log::MockNativeHostLogRepository;
        type WorkOmitRepo = domain::repositoryv2::work_omit::MockWorkOmitRepository;
        type DmmPackRepo = domain::repositoryv2::dmm_work_pack::MockDmmPackRepository;
        type DmmWorkRepo = domain::repositoryv2::works::MockDmmWorkRepository;
        type DlsiteWorkRepo = domain::repositoryv2::works::MockDlsiteWorkRepository;
        type WorkRepo = domain::repositoryv2::works::MockWorkRepository;
        type WorkParentPacksRepo = domain::repositoryv2::work_parent_packs::MockWorkParentPacksRepository;

        fn collection_repository(&self) -> &domain::repositoryv2::collection::MockCollectionRepository;
        fn explored_cache_repository(&self) -> &domain::repositoryv2::explored_cache::MockExploredCacheRepository;
        fn all_game_cache_repository(&self) -> &domain::repositoryv2::all_game_cache::MockAllGameCacheRepository;
        fn image_queue_repository(&self) -> &domain::repositoryv2::save_image_queue::MockImageSaveQueueRepository;
        fn host_log_repository(&self) -> &domain::repositoryv2::native_host_log::MockNativeHostLogRepository;
        fn work_omit_repository(&self) -> &domain::repositoryv2::work_omit::MockWorkOmitRepository;
        fn dmm_pack_repository(&self) -> &domain::repositoryv2::dmm_work_pack::MockDmmPackRepository;
        fn dmm_work_repository(&self) -> &domain::repositoryv2::works::MockDmmWorkRepository;
        fn dlsite_work_repository(&self) -> &domain::repositoryv2::works::MockDlsiteWorkRepository;
        fn work_repository(&self) -> &domain::repositoryv2::works::MockWorkRepository;
        fn work_parent_packs_repository(&self) -> &domain::repositoryv2::work_parent_packs::MockWorkParentPacksRepository;
    }
}

#[cfg(test)]
impl MockRepositoriesExtMock {
    pub fn with_default_work_repos(mut self) -> Self {
        use domain::repositoryv2::works::{MockDmmWorkRepository, MockDlsiteWorkRepository};
        use domain::works::{DmmWork, DlsiteWork};
        use domain::Id;
        use mockall::predicate::*;

        let mut dmm_repo = MockDmmWorkRepository::new();
        dmm_repo
            .expect_find_by_store_key()
            .with(always(), always(), always())
            .returning(|store_id, category, subcategory| {
                let s = store_id.to_string();
                let c = category.to_string();
                let sub = subcategory.to_string();
                Box::pin(async move {
                    Ok::<_, anyhow::Error>(Some(DmmWork {
                        id: Id::new(1000),
                        title: format!("{}-{}-{}", s, c, sub),
                        store_id: s,
                        category: c,
                        subcategory: sub,
                    }))
                })
            });
        dmm_repo
            .expect_find_by_store_keys()
            .with(always())
            .returning(|keys| {
                let mut id_counter = 1000i32;
                let list: Vec<domain::works::DmmWork> = keys
                    .iter()
                    .map(|(sid, cat, sub)| {
                        id_counter += 1;
                        domain::works::DmmWork {
                            id: domain::Id::new(id_counter),
                            title: format!("{}-{}-{}", sid, cat, sub),
                            store_id: sid.clone(),
                            category: cat.clone(),
                            subcategory: sub.clone(),
                        }
                    })
                    .collect();
                Box::pin(async move { Ok::<_, anyhow::Error>(list) })
            });
        self.expect_dmm_work().return_const(dmm_repo);

        let mut dl_repo = MockDlsiteWorkRepository::new();
        dl_repo
            .expect_find_by_store_key()
            .with(always(), always())
            .returning(|store_id, category| {
                let s = store_id.to_string();
                let c = category.to_string();
                Box::pin(async move {
                    Ok::<_, anyhow::Error>(Some(DlsiteWork {
                        id: Id::new(2000),
                        title: format!("{}-{}", s, c),
                        store_id: s,
                        category: c,
                    }))
                })
            });
        self.expect_dlsite_work().return_const(dl_repo);

        // WorkParentPacksRepository: デフォルトはno-opのモックを返す
        use domain::repositoryv2::work_parent_packs::MockWorkParentPacksRepository;
        let mut wpp = MockWorkParentPacksRepository::new();
        wpp.expect_add().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        wpp.expect_exists().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(false) }));
        self.expect_work_parent_packs().return_const(wpp);

        self
    }
}


