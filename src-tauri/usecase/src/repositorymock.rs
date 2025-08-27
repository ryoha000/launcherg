#[cfg(test)]
mockall::mock! {
    pub RepositoriesExtMock {}

    impl domain::repository::RepositoriesExt for RepositoriesExtMock {
        type CollectionRepo = domain::repository::collection::MockCollectionRepository;
        type ExploredCacheRepo = domain::repository::explored_cache::MockExploredCacheRepository;
        type AllGameCacheRepo = domain::repository::all_game_cache::MockAllGameCacheRepository;
        type ImageQueueRepo = domain::repository::save_image_queue::MockImageSaveQueueRepository;
        type HostLogRepo = domain::repository::native_host_log::MockNativeHostLogRepository;
        type WorkOmitRepo = domain::repository::work_omit::MockWorkOmitRepository;
        type DmmPackRepo = domain::repository::dmm_work_pack::MockDmmPackRepository;
        type DmmWorkRepo = domain::repository::works::MockDmmWorkRepository;
        type DlsiteWorkRepo = domain::repository::works::MockDlsiteWorkRepository;
        type WorkRepo = domain::repository::works::MockWorkRepository;
        type WorkParentPacksRepo = domain::repository::work_parent_packs::MockWorkParentPacksRepository;

        fn collection_repository(&self) -> &domain::repository::collection::MockCollectionRepository;
        fn explored_cache_repository(&self) -> &domain::repository::explored_cache::MockExploredCacheRepository;
        fn all_game_cache_repository(&self) -> &domain::repository::all_game_cache::MockAllGameCacheRepository;
        fn image_queue_repository(&self) -> &domain::repository::save_image_queue::MockImageSaveQueueRepository;
        fn host_log_repository(&self) -> &domain::repository::native_host_log::MockNativeHostLogRepository;
        fn work_omit_repository(&self) -> &domain::repository::work_omit::MockWorkOmitRepository;
        fn dmm_pack_repository(&self) -> &domain::repository::dmm_work_pack::MockDmmPackRepository;
        fn dmm_work_repository(&self) -> &domain::repository::works::MockDmmWorkRepository;
        fn dlsite_work_repository(&self) -> &domain::repository::works::MockDlsiteWorkRepository;
        fn work_repository(&self) -> &domain::repository::works::MockWorkRepository;
        fn work_parent_packs_repository(&self) -> &domain::repository::work_parent_packs::MockWorkParentPacksRepository;
    }
}

#[cfg(test)]
impl MockRepositoriesExtMock {
    pub fn with_default_work_repos(mut self) -> Self {
        use domain::repository::works::{MockDmmWorkRepository, MockDlsiteWorkRepository};
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
        self.expect_dmm_work_repository().return_const(dmm_repo);

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
        self.expect_dlsite_work_repository().return_const(dl_repo);

        // WorkParentPacksRepository: デフォルトはno-opのモックを返す
        use domain::repository::work_parent_packs::MockWorkParentPacksRepository;
        let mut wpp = MockWorkParentPacksRepository::new();
        wpp.expect_add().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        wpp.expect_exists().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(false) }));
        self.expect_work_parent_packs_repository().return_const(wpp);

        self
    }
}


