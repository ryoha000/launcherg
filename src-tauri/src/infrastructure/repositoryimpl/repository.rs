use derive_new::new;
use std::marker::PhantomData;

use crate::domain::{
    all_game_cache::AllGameCache,
    collection::CollectionElement,
    explored_cache::ExploredCache,
    repository::{
        all_game_cache::AllGameCacheRepository, collection::CollectionRepository,
        explored_cache::ExploredCacheRepository,
    },
};

use super::driver::Db;

#[derive(new)]
pub struct RepositoryImpl<T> {
    pub pool: Db,
    _marker: PhantomData<T>,
}

pub struct Repositories {
    collection_repository: RepositoryImpl<CollectionElement>,
    explored_cache_repository: RepositoryImpl<ExploredCache>,
    all_game_cache_repository: RepositoryImpl<AllGameCache>,
}
pub trait RepositoriesExt {
    type CollectionRepo: CollectionRepository;
    type ExploredCacheRepo: ExploredCacheRepository;
    type AllGameCacheRepo: AllGameCacheRepository;

    fn collection_repository(&self) -> &Self::CollectionRepo;
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo;
    fn all_game_cache_repository(&self) -> &Self::AllGameCacheRepo;
}

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

impl RepositoriesExt for Repositories {
    type CollectionRepo = RepositoryImpl<CollectionElement>;
    type ExploredCacheRepo = RepositoryImpl<ExploredCache>;
    type AllGameCacheRepo = RepositoryImpl<AllGameCache>;

    fn collection_repository(&self) -> &Self::CollectionRepo {
        &self.collection_repository
    }
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo {
        &self.explored_cache_repository
    }
    fn all_game_cache_repository(&self) -> &Self::AllGameCacheRepo {
        &self.all_game_cache_repository
    }
}

impl Repositories {
    pub fn new(db: Db) -> Self {
        let collection_repository = RepositoryImpl::new(db.clone());
        let explored_cache_repository = RepositoryImpl::new(db.clone());
        let all_game_cache_repository = RepositoryImpl::new(db.clone());

        Self {
            collection_repository,
            explored_cache_repository,
            all_game_cache_repository,
        }
    }
}
