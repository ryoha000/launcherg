pub mod all_game_cache;
pub mod collection;
pub mod explored_cache;
pub mod save_image_queue;
pub mod native_host_log;
pub mod work_omit;
pub mod dmm_work_pack;
pub mod works;
pub mod work_parent_packs;

pub trait RepositoriesExt {
    type CollectionRepo: collection::CollectionRepository;
    type ExploredCacheRepo: explored_cache::ExploredCacheRepository;
    type AllGameCacheRepo: all_game_cache::AllGameCacheRepository;
    type ImageQueueRepo: save_image_queue::ImageSaveQueueRepository;
    type HostLogRepo: native_host_log::NativeHostLogRepository;
    type WorkOmitRepo: work_omit::WorkOmitRepository;
    type WorkParentPacksRepo: work_parent_packs::WorkParentPacksRepository;
    type DmmPackRepo: dmm_work_pack::DmmPackRepository;
    type DmmWorkRepo: works::DmmWorkRepository;
    type DlsiteWorkRepo: works::DlsiteWorkRepository;
    type WorkRepo: works::WorkRepository;

    fn collection_repository(&self) -> &Self::CollectionRepo;
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo;
    fn all_game_cache_repository(&self) -> &Self::AllGameCacheRepo;
    fn image_queue_repository(&self) -> &Self::ImageQueueRepo;
    fn host_log_repository(&self) -> &Self::HostLogRepo;
    fn work_omit_repository(&self) -> &Self::WorkOmitRepo;
    fn work_parent_packs_repository(&self) -> &Self::WorkParentPacksRepo;
    fn dmm_pack_repository(&self) -> &Self::DmmPackRepo;
    fn dmm_work_repository(&self) -> &Self::DmmWorkRepo;
    fn dlsite_work_repository(&self) -> &Self::DlsiteWorkRepo;
    fn work_repository(&self) -> &Self::WorkRepo;
}
