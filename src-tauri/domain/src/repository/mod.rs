pub mod transaction;
pub mod works;
pub mod all_game_cache;
pub mod explored_cache;
pub mod save_image_queue;
pub mod native_host_log;
pub mod work_omit;
pub mod work_parent_packs;
pub mod dmm_work_pack;
pub mod collection;

pub trait RepositoriesExt {
    type WorkRepo: works::WorkRepository;
    type DmmWorkRepo: works::DmmWorkRepository;
    type DlsiteWorkRepo: works::DlsiteWorkRepository;
    type AllGameCacheRepo: all_game_cache::AllGameCacheRepository;
    type ExploredCacheRepo: explored_cache::ExploredCacheRepository;
    type ImageQueueRepo: save_image_queue::ImageSaveQueueRepository;
    type HostLogRepo: native_host_log::NativeHostLogRepository;
    type WorkOmitRepo: work_omit::WorkOmitRepository;
    type WorkParentPacksRepo: work_parent_packs::WorkParentPacksRepository;
    type DmmPackRepo: dmm_work_pack::DmmPackRepository;
    type CollectionRepo: collection::CollectionRepository;
    type TransactionRepo: transaction::TransactionRepository;

    fn work(&mut self) -> &mut Self::WorkRepo;
    fn dmm_work(&mut self) -> &mut Self::DmmWorkRepo;
    fn dlsite_work(&mut self) -> &mut Self::DlsiteWorkRepo;
    fn all_game_cache(&mut self) -> &mut Self::AllGameCacheRepo;
    fn explored_cache(&mut self) -> &mut Self::ExploredCacheRepo;
    fn image_queue(&mut self) -> &mut Self::ImageQueueRepo;
    fn host_log(&mut self) -> &mut Self::HostLogRepo;
    fn work_omit(&mut self) -> &mut Self::WorkOmitRepo;
    fn work_parent_packs(&mut self) -> &mut Self::WorkParentPacksRepo;
    fn dmm_pack(&mut self) -> &mut Self::DmmPackRepo;
    fn collection(&mut self) -> &mut Self::CollectionRepo;
    fn transaction(&mut self) -> &mut Self::TransactionRepo;
}
