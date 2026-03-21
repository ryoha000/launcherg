pub mod all_game_cache;
pub mod app_settings;
pub mod erogamescape;
pub mod explored_cache;
pub mod manager;
pub mod mock;
pub mod native_host_log;
pub mod save_image_queue;
pub mod work_download_path;
pub mod work_like;
pub mod work_lnk;
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
    type WorkParentPacksRepo: work_parent_packs::WorkParentPacksRepository;
    type ErogamescapeRepo: erogamescape::ErogamescapeRepository;
    type WorkDownloadPathRepo: work_download_path::WorkDownloadPathRepository;
    type WorkLnkRepo: work_lnk::WorkLnkRepository;
    type WorkLikeRepo: work_like::WorkLikeRepository;
    type WorkLinkPendingExeRepo: crate::work_link_pending_exe::WorkLinkPendingExeRepository;
    type AppSettingsRepo: app_settings::AppSettingsRepository;

    fn work(&self) -> Self::WorkRepo;
    fn dmm_work(&self) -> Self::DmmWorkRepo;
    fn dlsite_work(&self) -> Self::DlsiteWorkRepo;
    fn all_game_cache(&self) -> Self::AllGameCacheRepo;
    fn explored_cache(&self) -> Self::ExploredCacheRepo;
    fn image_queue(&self) -> Self::ImageQueueRepo;
    fn host_log(&self) -> Self::HostLogRepo;
    fn work_parent_packs(&self) -> Self::WorkParentPacksRepo;
    fn erogamescape(&self) -> Self::ErogamescapeRepo;
    fn work_download_path(&self) -> Self::WorkDownloadPathRepo;
    fn work_lnk(&self) -> Self::WorkLnkRepo;
    fn work_like(&self) -> Self::WorkLikeRepo;
    fn work_link_pending_exe(&self) -> Self::WorkLinkPendingExeRepo;
    fn app_settings(&self) -> Self::AppSettingsRepo;
}
