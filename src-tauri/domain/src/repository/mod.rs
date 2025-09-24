pub mod all_game_cache;
pub mod collection;
pub mod dmm_work_pack;
pub mod erogamescape;
pub mod explored_cache;
pub mod manager;
pub mod mock;
pub mod native_host_log;
pub mod save_image_queue;
pub mod work_download_path;
pub mod work_lnk;
pub mod work_omit;
pub mod work_parent_packs;
pub mod works;

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
    type ErogamescapeRepo: erogamescape::ErogamescapeRepository;
    type WorkDownloadPathRepo: work_download_path::WorkDownloadPathRepository;
    type WorkLnkRepo: work_lnk::WorkLnkRepository;

    fn work(&self) -> Self::WorkRepo;
    fn dmm_work(&self) -> Self::DmmWorkRepo;
    fn dlsite_work(&self) -> Self::DlsiteWorkRepo;
    fn all_game_cache(&self) -> Self::AllGameCacheRepo;
    fn explored_cache(&self) -> Self::ExploredCacheRepo;
    fn image_queue(&self) -> Self::ImageQueueRepo;
    fn host_log(&self) -> Self::HostLogRepo;
    fn work_omit(&self) -> Self::WorkOmitRepo;
    fn work_parent_packs(&self) -> Self::WorkParentPacksRepo;
    fn dmm_pack(&self) -> Self::DmmPackRepo;
    fn collection(&self) -> Self::CollectionRepo;
    fn erogamescape(&self) -> Self::ErogamescapeRepo;
    fn work_download_path(&self) -> Self::WorkDownloadPathRepo;
    fn work_lnk(&self) -> Self::WorkLnkRepo;
}
