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
        type TransactionRepo = Self;

        fn work(&mut self) -> &mut domain::repository::works::MockWorkRepository;
        fn dmm_work(&mut self) -> &mut domain::repository::works::MockDmmWorkRepository;
        fn dlsite_work(&mut self) -> &mut domain::repository::works::MockDlsiteWorkRepository;
        fn all_game_cache(&mut self) -> &mut domain::repository::all_game_cache::MockAllGameCacheRepository;
        fn explored_cache(&mut self) -> &mut domain::repository::explored_cache::MockExploredCacheRepository;
        fn image_queue(&mut self) -> &mut domain::repository::save_image_queue::MockImageSaveQueueRepository;
        fn host_log(&mut self) -> &mut domain::repository::native_host_log::MockNativeHostLogRepository;
        fn work_omit(&mut self) -> &mut domain::repository::work_omit::MockWorkOmitRepository;
        fn work_parent_packs(&mut self) -> &mut domain::repository::work_parent_packs::MockWorkParentPacksRepository;
        fn dmm_pack(&mut self) -> &mut domain::repository::dmm_work_pack::MockDmmPackRepository;
        fn collection(&mut self) -> &mut domain::repository::collection::MockCollectionRepository;
        fn transaction(&mut self) -> &mut Self;
    }
}

#[cfg(test)]
impl domain::repository::transaction::TransactionRepository for MockRepositoriesExtMock {
    async fn with_transaction<F, R>(&mut self, f: F) -> anyhow::Result<R>
    where
        for<'cx> F: FnOnce(&'cx mut Self) -> futures::future::BoxFuture<'cx, anyhow::Result<R>> + Send,
        R: Send,
    {
        f(self).await
    }
}

#[cfg(test)]
impl MockRepositoriesExtMock {
    pub fn with_default_work_repos(mut self) -> Self {
        self
    }
}

#[cfg(test)]
pub struct TestRepositories {
    pub collection: domain::repository::collection::MockCollectionRepository,
    pub explored_cache: domain::repository::explored_cache::MockExploredCacheRepository,
    pub all_game_cache: domain::repository::all_game_cache::MockAllGameCacheRepository,
    pub image_queue: domain::repository::save_image_queue::MockImageSaveQueueRepository,
    pub host_log: domain::repository::native_host_log::MockNativeHostLogRepository,
    pub work_omit: domain::repository::work_omit::MockWorkOmitRepository,
    pub dmm_pack: domain::repository::dmm_work_pack::MockDmmPackRepository,
    pub dmm_work: domain::repository::works::MockDmmWorkRepository,
    pub dlsite_work: domain::repository::works::MockDlsiteWorkRepository,
    pub work: domain::repository::works::MockWorkRepository,
    pub work_parent_packs: domain::repository::work_parent_packs::MockWorkParentPacksRepository,
}

#[cfg(test)]
impl Default for TestRepositories {
    fn default() -> Self {
        Self {
            collection: Default::default(),
            explored_cache: Default::default(),
            all_game_cache: Default::default(),
            image_queue: Default::default(),
            host_log: Default::default(),
            work_omit: Default::default(),
            dmm_pack: Default::default(),
            dmm_work: Default::default(),
            dlsite_work: Default::default(),
            work: Default::default(),
            work_parent_packs: Default::default(),
        }
    }
}

#[cfg(test)]
impl domain::repository::RepositoriesExt for TestRepositories {
    type WorkRepo = domain::repository::works::MockWorkRepository;
    type DmmWorkRepo = domain::repository::works::MockDmmWorkRepository;
    type DlsiteWorkRepo = domain::repository::works::MockDlsiteWorkRepository;
    type AllGameCacheRepo = domain::repository::all_game_cache::MockAllGameCacheRepository;
    type ExploredCacheRepo = domain::repository::explored_cache::MockExploredCacheRepository;
    type ImageQueueRepo = domain::repository::save_image_queue::MockImageSaveQueueRepository;
    type HostLogRepo = domain::repository::native_host_log::MockNativeHostLogRepository;
    type WorkOmitRepo = domain::repository::work_omit::MockWorkOmitRepository;
    type WorkParentPacksRepo = domain::repository::work_parent_packs::MockWorkParentPacksRepository;
    type DmmPackRepo = domain::repository::dmm_work_pack::MockDmmPackRepository;
    type CollectionRepo = domain::repository::collection::MockCollectionRepository;
    type TransactionRepo = MockRepositoriesExtMock; // 未使用だが型を満たすため

    fn work(&mut self) -> &mut Self::WorkRepo { &mut self.work }
    fn dmm_work(&mut self) -> &mut Self::DmmWorkRepo { &mut self.dmm_work }
    fn dlsite_work(&mut self) -> &mut Self::DlsiteWorkRepo { &mut self.dlsite_work }
    fn all_game_cache(&mut self) -> &mut Self::AllGameCacheRepo { &mut self.all_game_cache }
    fn explored_cache(&mut self) -> &mut Self::ExploredCacheRepo { &mut self.explored_cache }
    fn image_queue(&mut self) -> &mut Self::ImageQueueRepo { &mut self.image_queue }
    fn host_log(&mut self) -> &mut Self::HostLogRepo { &mut self.host_log }
    fn work_omit(&mut self) -> &mut Self::WorkOmitRepo { &mut self.work_omit }
    fn work_parent_packs(&mut self) -> &mut Self::WorkParentPacksRepo { &mut self.work_parent_packs }
    fn dmm_pack(&mut self) -> &mut Self::DmmPackRepo { &mut self.dmm_pack }
    fn collection(&mut self) -> &mut Self::CollectionRepo { &mut self.collection }
    fn transaction(&mut self) -> &mut Self::TransactionRepo { unimplemented!("Transaction mock not used in these tests") }
}

#[cfg(test)]
impl TestRepositories {
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
        self.dmm_work = dmm_repo;

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
        self.dlsite_work = dl_repo;

        use domain::repository::work_parent_packs::MockWorkParentPacksRepository;
        let mut wpp = MockWorkParentPacksRepository::new();
        wpp.expect_add().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(()) }));
        wpp.expect_exists().returning(|_, _| Box::pin(async { Ok::<_, anyhow::Error>(false) }));
        self.work_parent_packs = wpp;

        self
    }

    pub fn set_all_game_cache(&mut self, repo: domain::repository::all_game_cache::MockAllGameCacheRepository) {
        self.all_game_cache = repo;
    }
}


