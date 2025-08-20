#[cfg(any(test, feature = "mocks"))]
mockall::mock! {
    pub RepositoriesExtMock {}
    
    impl super::repositoryimpl::repository::RepositoriesExt for RepositoriesExtMock {
        type CollectionRepo = crate::domain::repository::collection::MockCollectionRepository;
        type ExploredCacheRepo = crate::domain::repository::explored_cache::MockExploredCacheRepository;
        type AllGameCacheRepo = crate::domain::repository::all_game_cache::MockAllGameCacheRepository;
        type ImageQueueRepo = crate::domain::repository::save_image_queue::MockImageSaveQueueRepository;
        type HostLogRepo = crate::domain::repository::native_host_log::MockNativeHostLogRepository;

        fn collection_repository(&self) -> &crate::domain::repository::collection::MockCollectionRepository;
        fn explored_cache_repository(&self) -> &crate::domain::repository::explored_cache::MockExploredCacheRepository;
        fn all_game_cache_repository(&self) -> &crate::domain::repository::all_game_cache::MockAllGameCacheRepository;
        fn image_queue_repository(&self) -> &crate::domain::repository::save_image_queue::MockImageSaveQueueRepository;
        fn host_log_repository(&self) -> &crate::domain::repository::native_host_log::MockNativeHostLogRepository;
    }
}