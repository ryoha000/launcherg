use crate::domain::repository::{
    all_game_cache::AllGameCacheRepository, collection::CollectionRepository,
    explored_cache::ExploredCacheRepository,
};

use super::repositoryimpl::repository::RepositoriesExt;

#[cfg(test)]
mockall::mock! {
    pub RepositoriesExtMock {}
    impl RepositoriesExt for RepositoriesExtMock {
        type CollectionRepo = crate::domain::repository::collection::MockCollectionRepository;
        type ExploredCacheRepo = crate::domain::repository::explored_cache::MockExploredCacheRepository;
        type AllGameCacheRepo = crate::domain::repository::all_game_cache::MockAllGameCacheRepository;

        fn collection_repository(&self) -> &Self::CollectionRepo;
        fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo;
        fn all_game_cache_repository(&self) -> &Self::AllGameCacheRepo;
    }
}