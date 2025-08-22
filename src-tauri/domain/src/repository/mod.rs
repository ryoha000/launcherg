pub mod all_game_cache;
pub mod collection;
pub mod explored_cache;
pub mod save_image_queue;
pub mod native_host_log;
pub mod deny_list;
pub mod dmm_pack;

pub trait RepositoriesExt {
    type CollectionRepo: collection::CollectionRepository;
    type ExploredCacheRepo: explored_cache::ExploredCacheRepository;
    type AllGameCacheRepo: all_game_cache::AllGameCacheRepository;
    type ImageQueueRepo: save_image_queue::ImageSaveQueueRepository;
    type HostLogRepo: native_host_log::NativeHostLogRepository;
    type DenyListRepo: deny_list::DenyListRepository;
    type DmmPackRepo: dmm_pack::DmmPackRepository;

    fn collection_repository(&self) -> &Self::CollectionRepo;
    fn explored_cache_repository(&self) -> &Self::ExploredCacheRepo;
    fn all_game_cache_repository(&self) -> &Self::AllGameCacheRepo;
    fn image_queue_repository(&self) -> &Self::ImageQueueRepo;
    fn host_log_repository(&self) -> &Self::HostLogRepo;
    fn deny_list_repository(&self) -> &Self::DenyListRepo;
    fn dmm_pack_repository(&self) -> &Self::DmmPackRepo;
}
