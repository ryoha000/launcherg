use derive_new::new;
use domain::{native_host_log::NativeHostLogRow, repository::RepositoriesExt, save_image_queue::ImageSaveQueueRow, works::{DmmWork, DlsiteWork, Work}};
use std::marker::PhantomData;

use domain::{
    all_game_cache::AllGameCache,
    collection::CollectionElement,
    explored_cache::ExploredCache,
    work_omit::{WorkOmit},
    dmm_work_pack::DmmWorkPack,
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
    work_omit_repository: RepositoryImpl<WorkOmit>,
    dmm_pack_repository: RepositoryImpl<DmmWorkPack>,
    dmm_work_repository: RepositoryImpl<DmmWork>,
    dlsite_work_repository: RepositoryImpl<DlsiteWork>,
    work_repository: RepositoryImpl<Work>,
}

impl RepositoriesExt for Repositories {
    type CollectionRepo = RepositoryImpl<CollectionElement>;
    type ExploredCacheRepo = RepositoryImpl<ExploredCache>;
    type AllGameCacheRepo = RepositoryImpl<AllGameCache>;
    type ImageQueueRepo = RepositoryImpl<ImageSaveQueueRow>;
    type HostLogRepo = RepositoryImpl<NativeHostLogRow>;
    type WorkOmitRepo = RepositoryImpl<WorkOmit>;
    type DmmPackRepo = RepositoryImpl<DmmWorkPack>;
    type DmmWorkRepo = RepositoryImpl<DmmWork>;
    type DlsiteWorkRepo = RepositoryImpl<DlsiteWork>;
    type WorkRepo = RepositoryImpl<Work>;

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
    fn work_omit_repository(&self) -> &Self::WorkOmitRepo { &self.work_omit_repository }
    fn dmm_pack_repository(&self) -> &Self::DmmPackRepo { &self.dmm_pack_repository }
    fn dmm_work_repository(&self) -> &Self::DmmWorkRepo { &self.dmm_work_repository }
    fn dlsite_work_repository(&self) -> &Self::DlsiteWorkRepo { &self.dlsite_work_repository }
    fn work_repository(&self) -> &Self::WorkRepo { &self.work_repository }
}

impl Repositories {
    pub fn new(db: Db) -> Self {
        let collection_repository = RepositoryImpl::new(db.clone());
        let explored_cache_repository = RepositoryImpl::new(db.clone());
        let all_game_cache_repository = RepositoryImpl::new(db.clone());
        let image_queue_repository = RepositoryImpl::new(db.clone());
        let host_log_repository = RepositoryImpl::new(db.clone());
        let work_omit_repository = RepositoryImpl::new(db.clone());
        let dmm_pack_repository = RepositoryImpl::new(db.clone());
        let dmm_work_repository = RepositoryImpl::new(db.clone());
        let dlsite_work_repository = RepositoryImpl::new(db.clone());
        let work_repository = RepositoryImpl::new(db.clone());

        Self {
            collection_repository,
            explored_cache_repository,
            all_game_cache_repository,
            image_queue_repository,
            host_log_repository,
            work_omit_repository,
            dmm_pack_repository,
            dmm_work_repository,
            dlsite_work_repository,
            work_repository,
        }
    }

    pub fn pool(&self) -> &Db {
        &self.collection_repository.pool
    }
}
