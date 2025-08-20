use derive_new::new;
use domain::{native_host_log::NativeHostLogRow, save_image_queue::ImageSaveQueueRow};
use std::marker::PhantomData;

use crate::domain::{
    all_game_cache::AllGameCache,
    collection::CollectionElement,
    explored_cache::ExploredCache,
    repository::{
        all_game_cache::AllGameCacheRepository, collection::CollectionRepository,
        explored_cache::ExploredCacheRepository,
        save_image_queue::ImageSaveQueueRepository,
        native_host_log::NativeHostLogRepository,
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
    image_queue_repository: RepositoryImpl<ImageSaveQueueRow>,
    host_log_repository: RepositoryImpl<NativeHostLogRow>,
}
pub trait RepositoriesExt {
    type CollectionRepo: CollectionRepository;
    type ExploredCacheRepo: ExploredCacheRepository;
    type AllGameCacheRepo: AllGameCacheRepository;
    type ImageQueueRepo: ImageSaveQueueRepository;
    type HostLogRepo: NativeHostLogRepository;

    fn collection_repository(&self) -> &Self::CollectionRepo;
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo;
    fn all_game_cache_repository(&self) -> &Self::AllGameCacheRepo;
    fn image_queue_repository(&self) -> &Self::ImageQueueRepo;
    fn host_log_repository(&self) -> &Self::HostLogRepo;
}


impl RepositoriesExt for Repositories {
    type CollectionRepo = RepositoryImpl<CollectionElement>;
    type ExploredCacheRepo = RepositoryImpl<ExploredCache>;
    type AllGameCacheRepo = RepositoryImpl<AllGameCache>;
    type ImageQueueRepo = RepositoryImpl<ImageSaveQueueRow>;
    type HostLogRepo = RepositoryImpl<NativeHostLogRow>;

    fn collection_repository(&self) -> &Self::CollectionRepo {
        &self.collection_repository
    }
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo {
        &self.explored_cache_repository
    }
    fn all_game_cache_repository(&self) -> &Self::AllGameCacheRepo {
        &self.all_game_cache_repository
    }
    fn image_queue_repository(&self) -> &Self::ImageQueueRepo { &self.image_queue_repository }
    fn host_log_repository(&self) -> &Self::HostLogRepo { &self.host_log_repository }
}

impl Repositories {
    pub fn new(db: Db) -> Self {
        let collection_repository = RepositoryImpl::new(db.clone());
        let explored_cache_repository = RepositoryImpl::new(db.clone());
        let all_game_cache_repository = RepositoryImpl::new(db.clone());
        let image_queue_repository = RepositoryImpl::new(db.clone());
        let host_log_repository = RepositoryImpl::new(db.clone());

        Self {
            collection_repository,
            explored_cache_repository,
            all_game_cache_repository,
            image_queue_repository,
            host_log_repository,
        }
    }

    pub fn pool(&self) -> &Db {
        &self.collection_repository.pool
    }
}
