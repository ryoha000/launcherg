#[cfg(test)]
mockall::mock! {
    pub RepositoriesExtMock {}

    impl infrastructure::repositoryimpl::repository::RepositoriesExt for RepositoriesExtMock {
        type CollectionRepo = domain::repository::collection::MockCollectionRepository;
        type ExploredCacheRepo = domain::repository::explored_cache::MockExploredCacheRepository;
        type AllGameCacheRepo = domain::repository::all_game_cache::MockAllGameCacheRepository;
        type ImageQueueRepo = domain::repository::save_image_queue::MockImageSaveQueueRepository;
        type HostLogRepo = domain::repository::native_host_log::MockNativeHostLogRepository;

        fn collection_repository(&self) -> &domain::repository::collection::MockCollectionRepository;
        fn explored_cache_repository(&self) -> &domain::repository::explored_cache::MockExploredCacheRepository;
        fn all_game_cache_repository(&self) -> &domain::repository::all_game_cache::MockAllGameCacheRepository;
        fn image_queue_repository(&self) -> &domain::repository::save_image_queue::MockImageSaveQueueRepository;
        fn host_log_repository(&self) -> &domain::repository::native_host_log::MockNativeHostLogRepository;
    }
}


