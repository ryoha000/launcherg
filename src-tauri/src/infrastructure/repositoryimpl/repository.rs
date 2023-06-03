use derive_new::new;
use std::marker::PhantomData;

use crate::domain::{
    collection::Collection,
    explored_cache::ExploredCache,
    repository::{collection::CollectionRepository, explored_cache::ExploredCacheRepository},
};

use super::driver::Db;

#[derive(new)]
pub struct RepositoryImpl<T> {
    pub pool: Db,
    _marker: PhantomData<T>,
}

pub struct Repositories {
    collection_repository: RepositoryImpl<Collection>,
    explored_cache_repository: RepositoryImpl<ExploredCache>,
}
pub trait RepositoriesExt {
    type CollectionRepo: CollectionRepository;
    type ExploredCacheRepo: ExploredCacheRepository;

    fn collection_repository(&self) -> &Self::CollectionRepo;
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo;
}

impl RepositoriesExt for Repositories {
    type CollectionRepo = RepositoryImpl<Collection>;
    type ExploredCacheRepo = RepositoryImpl<ExploredCache>;

    fn collection_repository(&self) -> &Self::CollectionRepo {
        &self.collection_repository
    }
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo {
        &self.explored_cache_repository
    }
}

impl Repositories {
    pub fn new(db: Db) -> Self {
        let collection_repository = RepositoryImpl::new(db.clone());
        let explored_cache_repository = RepositoryImpl::new(db.clone());

        Self {
            collection_repository,
            explored_cache_repository,
        }
    }
}
