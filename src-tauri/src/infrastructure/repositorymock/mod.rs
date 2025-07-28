#[cfg(test)]
mockall::mock! {
    pub RepositoriesExtMock {}
    
    impl super::repositoryimpl::repository::RepositoriesExt for RepositoriesExtMock {
        type CollectionRepo = crate::domain::repository::collection::MockCollectionRepository;
        type ExploredCacheRepo = crate::domain::repository::explored_cache::MockExploredCacheRepository;
        type AllGameCacheRepo = crate::domain::repository::all_game_cache::MockAllGameCacheRepository;

        fn collection_repository(&self) -> &crate::domain::repository::collection::MockCollectionRepository;
        fn explored_cache_repository(&self) -> &crate::domain::repository::explored_cache::MockExploredCacheRepository;
        fn all_game_cache_repository(&self) -> &crate::domain::repository::all_game_cache::MockAllGameCacheRepository;
    }
}