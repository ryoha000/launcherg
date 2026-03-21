use std::sync::Arc;
use tauri::async_runtime::Mutex;

#[cfg(test)]
mockall::mock! {
    pub RepositoriesExtMock {}

    impl domain::repository::RepositoriesExt for RepositoriesExtMock {
        type ExploredCacheRepo = domain::repository::explored_cache::MockExploredCacheRepository;
        type AllGameCacheRepo = domain::repository::all_game_cache::MockAllGameCacheRepository;
        type AppSettingsRepo = domain::repository::app_settings::MockAppSettingsRepository;
        type ImageQueueRepo = domain::repository::save_image_queue::MockImageSaveQueueRepository;
        type HostLogRepo = domain::repository::native_host_log::MockNativeHostLogRepository;
        type DmmWorkRepo = domain::repository::works::MockDmmWorkRepository;
        type DlsiteWorkRepo = domain::repository::works::MockDlsiteWorkRepository;
        type WorkRepo = domain::repository::works::MockWorkRepository;
        type WorkParentPacksRepo = domain::repository::work_parent_packs::MockWorkParentPacksRepository;
        type WorkDownloadPathRepo = domain::repository::work_download_path::MockWorkDownloadPathRepository;
        type WorkLnkRepo = domain::repository::work_lnk::MockWorkLnkRepository;
        type WorkLikeRepo = domain::repository::work_like::MockWorkLikeRepository;
        type WorkLinkPendingExeRepo = domain::work_link_pending_exe::MockWorkLinkPendingExeRepository;
        type ErogamescapeRepo = domain::repository::erogamescape::MockErogamescapeRepository;
        fn work(&self) -> domain::repository::works::MockWorkRepository;
        fn dmm_work(&self) -> domain::repository::works::MockDmmWorkRepository;
        fn dlsite_work(&self) -> domain::repository::works::MockDlsiteWorkRepository;
        fn all_game_cache(&self) -> domain::repository::all_game_cache::MockAllGameCacheRepository;
        fn app_settings(&self) -> domain::repository::app_settings::MockAppSettingsRepository;
        fn explored_cache(&self) -> domain::repository::explored_cache::MockExploredCacheRepository;
        fn image_queue(&self) -> domain::repository::save_image_queue::MockImageSaveQueueRepository;
        fn host_log(&self) -> domain::repository::native_host_log::MockNativeHostLogRepository;
        fn work_parent_packs(&self) -> domain::repository::work_parent_packs::MockWorkParentPacksRepository;
        fn work_download_path(&self) -> domain::repository::work_download_path::MockWorkDownloadPathRepository;
        fn work_lnk(&self) -> domain::repository::work_lnk::MockWorkLnkRepository;
        fn work_like(&self) -> domain::repository::work_like::MockWorkLikeRepository;
        fn work_link_pending_exe(&self) -> domain::work_link_pending_exe::MockWorkLinkPendingExeRepository;
        fn erogamescape(&self) -> domain::repository::erogamescape::MockErogamescapeRepository;
    }
}

#[cfg(test)]
#[derive(Clone)]
pub struct TestRepositories {
    pub explored_cache: Arc<Mutex<domain::repository::explored_cache::MockExploredCacheRepository>>,
    pub all_game_cache: Arc<Mutex<domain::repository::all_game_cache::MockAllGameCacheRepository>>,
    pub app_settings: Arc<Mutex<domain::repository::app_settings::MockAppSettingsRepository>>,
    pub image_queue: Arc<Mutex<domain::repository::save_image_queue::MockImageSaveQueueRepository>>,
    pub host_log: Arc<Mutex<domain::repository::native_host_log::MockNativeHostLogRepository>>,
    pub dmm_work: Arc<Mutex<domain::repository::works::MockDmmWorkRepository>>,
    pub dlsite_work: Arc<Mutex<domain::repository::works::MockDlsiteWorkRepository>>,
    pub work: Arc<Mutex<domain::repository::works::MockWorkRepository>>,
    pub work_parent_packs:
        Arc<Mutex<domain::repository::work_parent_packs::MockWorkParentPacksRepository>>,
    pub work_download_path:
        Arc<Mutex<domain::repository::work_download_path::MockWorkDownloadPathRepository>>,
    pub work_lnk: Arc<Mutex<domain::repository::work_lnk::MockWorkLnkRepository>>,
    pub work_like: Arc<Mutex<domain::repository::work_like::MockWorkLikeRepository>>,
    pub work_link_pending_exe:
        Arc<Mutex<domain::work_link_pending_exe::MockWorkLinkPendingExeRepository>>,
    pub erogamescape: Arc<Mutex<domain::repository::erogamescape::MockErogamescapeRepository>>,
}

#[cfg(test)]
impl Default for TestRepositories {
    fn default() -> Self {
        Self {
            explored_cache: Arc::new(Mutex::new(Default::default())),
            all_game_cache: Arc::new(Mutex::new(Default::default())),
            app_settings: Arc::new(Mutex::new(Default::default())),
            image_queue: Arc::new(Mutex::new(Default::default())),
            host_log: Arc::new(Mutex::new(Default::default())),
            dmm_work: Arc::new(Mutex::new(Default::default())),
            dlsite_work: Arc::new(Mutex::new(Default::default())),
            work: Arc::new(Mutex::new(Default::default())),
            work_parent_packs: Arc::new(Mutex::new(Default::default())),
            work_download_path: Arc::new(Mutex::new(Default::default())),
            work_lnk: Arc::new(Mutex::new(Default::default())),
            work_like: Arc::new(Mutex::new(Default::default())),
            work_link_pending_exe: Arc::new(Mutex::new(Default::default())),
            erogamescape: Arc::new(Mutex::new(Default::default())),
        }
    }
}

#[cfg(test)]
impl domain::repository::RepositoriesExt for TestRepositories {
    type WorkRepo = TestRepositories;
    type DmmWorkRepo = TestRepositories;
    type DlsiteWorkRepo = TestRepositories;
    type AllGameCacheRepo = TestRepositories;
    type AppSettingsRepo = TestRepositories;
    type ExploredCacheRepo = TestRepositories;
    type ImageQueueRepo = TestRepositories;
    type HostLogRepo = TestRepositories;
    type WorkParentPacksRepo = TestRepositories;
    type ErogamescapeRepo = TestRepositories;
    type WorkDownloadPathRepo = TestRepositories;
    type WorkLnkRepo = TestRepositories;
    type WorkLikeRepo = TestRepositories;
    type WorkLinkPendingExeRepo = TestRepositories;
    fn work(&self) -> Self::WorkRepo {
        self.clone()
    }
    fn dmm_work(&self) -> Self::DmmWorkRepo {
        self.clone()
    }
    fn dlsite_work(&self) -> Self::DlsiteWorkRepo {
        self.clone()
    }
    fn all_game_cache(&self) -> Self::AllGameCacheRepo {
        self.clone()
    }
    fn app_settings(&self) -> Self::AppSettingsRepo {
        self.clone()
    }
    fn explored_cache(&self) -> Self::ExploredCacheRepo {
        self.clone()
    }
    fn image_queue(&self) -> Self::ImageQueueRepo {
        self.clone()
    }
    fn host_log(&self) -> Self::HostLogRepo {
        self.clone()
    }
    fn work_parent_packs(&self) -> Self::WorkParentPacksRepo {
        self.clone()
    }
    fn erogamescape(&self) -> Self::ErogamescapeRepo {
        self.clone()
    }
    fn work_download_path(&self) -> Self::WorkDownloadPathRepo {
        self.clone()
    }
    fn work_lnk(&self) -> Self::WorkLnkRepo {
        self.clone()
    }
    fn work_like(&self) -> Self::WorkLikeRepo {
        self.clone()
    }
    fn work_link_pending_exe(&self) -> Self::WorkLinkPendingExeRepo {
        self.clone()
    }
}

#[cfg(test)]
impl domain::repository::works::WorkRepository for TestRepositories {
    async fn upsert(
        &mut self,
        new_work: &domain::works::NewWork,
    ) -> anyhow::Result<domain::StrId<domain::works::Work>> {
        self.work.lock().await.upsert(new_work).await
    }
    async fn find_by_title(&mut self, title: &str) -> anyhow::Result<Option<domain::works::Work>> {
        self.work.lock().await.find_by_title(title).await
    }
    async fn list_all_details(&mut self) -> anyhow::Result<Vec<domain::works::WorkDetails>> {
        self.work.lock().await.list_all_details().await
    }
    async fn find_details_by_work_id(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
    ) -> anyhow::Result<Option<domain::works::WorkDetails>> {
        self.work
            .lock()
            .await
            .find_details_by_work_id(work_id)
            .await
    }
    async fn find_work_ids_by_erogamescape_ids(
        &mut self,
        erogamescape_ids: &[i32],
    ) -> anyhow::Result<Vec<(i32, domain::StrId<domain::works::Work>)>> {
        self.work
            .lock()
            .await
            .find_work_ids_by_erogamescape_ids(erogamescape_ids)
            .await
    }
    async fn upsert_erogamescape_map(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
        erogamescape_id: i32,
    ) -> anyhow::Result<()> {
        self.work
            .lock()
            .await
            .upsert_erogamescape_map(work_id, erogamescape_id)
            .await
    }

    async fn delete(&mut self, id: domain::StrId<domain::works::Work>) -> anyhow::Result<()> {
        self.work.lock().await.delete(id).await
    }

    async fn list_work_ids_missing_thumbnail_size(
        &mut self,
    ) -> anyhow::Result<Vec<domain::StrId<domain::works::Work>>> {
        self.work
            .lock()
            .await
            .list_work_ids_missing_thumbnail_size()
            .await
    }

    async fn upsert_work_thumbnail_size(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
        width: i32,
        height: i32,
    ) -> anyhow::Result<()> {
        self.work
            .lock()
            .await
            .upsert_work_thumbnail_size(work_id, width, height)
            .await
    }

    async fn update_last_play_at_by_work_id(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
        last_play_at: chrono::DateTime<chrono::Local>,
    ) -> anyhow::Result<()> {
        self.work
            .lock()
            .await
            .update_last_play_at_by_work_id(work_id, last_play_at)
            .await
    }
    async fn update_install_by_work_id(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
        install_at: chrono::DateTime<chrono::Local>,
        original_path: String,
    ) -> anyhow::Result<()> {
        self.work
            .lock()
            .await
            .update_install_by_work_id(work_id, install_at, original_path)
            .await
    }
}

impl domain::repository::erogamescape::ErogamescapeRepository for TestRepositories {
    async fn upsert_information(
        &mut self,
        info: &domain::erogamescape::NewErogamescapeInformation,
    ) -> anyhow::Result<()> {
        self.erogamescape
            .lock()
            .await
            .upsert_information(info)
            .await
    }
    async fn find_missing_information_ids(&mut self) -> anyhow::Result<Vec<i32>> {
        self.erogamescape
            .lock()
            .await
            .find_missing_information_ids()
            .await
    }
}

#[cfg(test)]
impl domain::repository::works::DmmWorkRepository for TestRepositories {
    async fn upsert(
        &mut self,
        new_work: &domain::works::NewDmmWork,
    ) -> anyhow::Result<domain::Id<domain::works::DmmWork>> {
        self.dmm_work.lock().await.upsert(new_work).await
    }
    async fn find_by_store_key(
        &mut self,
        store_id: &str,
        category: &str,
        subcategory: &str,
    ) -> anyhow::Result<Option<domain::works::DmmWork>> {
        self.dmm_work
            .lock()
            .await
            .find_by_store_key(store_id, category, subcategory)
            .await
    }
    async fn find_by_store_id(
        &mut self,
        store_id: &str,
    ) -> anyhow::Result<Option<domain::works::DmmWork>> {
        self.dmm_work.lock().await.find_by_store_id(store_id).await
    }
    async fn find_by_store_keys(
        &mut self,
        keys: &[(String, String, String)],
    ) -> anyhow::Result<Vec<domain::works::DmmWork>> {
        self.dmm_work.lock().await.find_by_store_keys(keys).await
    }
    async fn find_by_work_id(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
    ) -> anyhow::Result<Option<domain::works::DmmWork>> {
        self.dmm_work.lock().await.find_by_work_id(work_id).await
    }
}

#[cfg(test)]
impl domain::repository::works::DlsiteWorkRepository for TestRepositories {
    async fn upsert(
        &mut self,
        new_work: &domain::works::NewDlsiteWork,
    ) -> anyhow::Result<domain::Id<domain::works::DlsiteWork>> {
        self.dlsite_work.lock().await.upsert(new_work).await
    }
    async fn find_by_store_key(
        &mut self,
        store_id: &str,
        category: &str,
    ) -> anyhow::Result<Option<domain::works::DlsiteWork>> {
        self.dlsite_work
            .lock()
            .await
            .find_by_store_key(store_id, category)
            .await
    }
    async fn find_by_store_id(
        &mut self,
        store_id: &str,
    ) -> anyhow::Result<Option<domain::works::DlsiteWork>> {
        self.dlsite_work
            .lock()
            .await
            .find_by_store_id(store_id)
            .await
    }
}

#[cfg(test)]
impl domain::repository::all_game_cache::AllGameCacheRepository for TestRepositories {
    async fn get_by_ids(
        &mut self,
        ids: Vec<i32>,
    ) -> anyhow::Result<Vec<domain::all_game_cache::AllGameCacheOneWithThumbnailUrl>> {
        self.all_game_cache.lock().await.get_by_ids(ids).await
    }
    async fn get_all(&mut self) -> anyhow::Result<domain::all_game_cache::AllGameCache> {
        self.all_game_cache.lock().await.get_all().await
    }
    async fn get_last_updated(&mut self) -> anyhow::Result<(i32, chrono::DateTime<chrono::Local>)> {
        self.all_game_cache.lock().await.get_last_updated().await
    }
    async fn delete_by_ids(&mut self, ids: Vec<i32>) -> anyhow::Result<()> {
        self.all_game_cache.lock().await.delete_by_ids(ids).await
    }
    async fn update(
        &mut self,
        cache: Vec<domain::all_game_cache::NewAllGameCacheOne>,
    ) -> anyhow::Result<()> {
        self.all_game_cache.lock().await.update(cache).await
    }
    async fn search_by_name(
        &mut self,
        name: &str,
    ) -> anyhow::Result<Vec<domain::all_game_cache::AllGameCacheOneWithThumbnailUrl>> {
        self.all_game_cache.lock().await.search_by_name(name).await
    }
}

impl domain::repository::app_settings::AppSettingsRepository for TestRepositories {
    async fn get_storage_settings(
        &mut self,
    ) -> anyhow::Result<domain::repository::app_settings::AppStorageSettings> {
        self.app_settings.lock().await.get_storage_settings().await
    }
    async fn set_storage_settings(
        &mut self,
        settings: &domain::repository::app_settings::AppStorageSettings,
    ) -> anyhow::Result<()> {
        self.app_settings.lock().await.set_storage_settings(settings).await
    }
}

#[cfg(test)]
impl domain::repository::explored_cache::ExploredCacheRepository for TestRepositories {
    async fn get_all(&mut self) -> anyhow::Result<domain::explored_cache::ExploredCache> {
        self.explored_cache.lock().await.get_all().await
    }
    async fn add(&mut self, adding: domain::explored_cache::ExploredCache) -> anyhow::Result<()> {
        self.explored_cache.lock().await.add(adding).await
    }
}

#[cfg(test)]
impl domain::repository::save_image_queue::ImageSaveQueueRepository for TestRepositories {
    async fn enqueue(
        &mut self,
        image_url: &str,
        src_type: domain::save_image_queue::ImageSrcType,
        dst_path: &str,
        preprocess: domain::save_image_queue::ImagePreprocess,
    ) -> anyhow::Result<domain::Id<domain::save_image_queue::ImageSaveQueueRow>> {
        self.image_queue
            .lock()
            .await
            .enqueue(image_url, src_type, dst_path, preprocess)
            .await
    }
    async fn list(
        &mut self,
        unfinished: bool,
        limit: i64,
    ) -> anyhow::Result<Vec<domain::save_image_queue::ImageSaveQueueRow>> {
        self.image_queue.lock().await.list(unfinished, limit).await
    }
    async fn count(&mut self, unfinished: bool) -> anyhow::Result<i64> {
        self.image_queue.lock().await.count(unfinished).await
    }
    async fn mark_finished(
        &mut self,
        id: domain::Id<domain::save_image_queue::ImageSaveQueueRow>,
    ) -> anyhow::Result<()> {
        self.image_queue.lock().await.mark_finished(id).await
    }
    async fn mark_failed(
        &mut self,
        id: domain::Id<domain::save_image_queue::ImageSaveQueueRow>,
        error: &str,
    ) -> anyhow::Result<()> {
        self.image_queue.lock().await.mark_failed(id, error).await
    }
}

#[cfg(test)]
impl domain::repository::native_host_log::NativeHostLogRepository for TestRepositories {
    async fn insert_log(
        &mut self,
        level: domain::native_host_log::HostLogLevel,
        typ: domain::native_host_log::HostLogType,
        message: &str,
    ) -> anyhow::Result<()> {
        self.host_log
            .lock()
            .await
            .insert_log(level, typ, message)
            .await
    }
    async fn list_logs(
        &mut self,
        limit: i64,
        offset: i64,
        level: Option<domain::native_host_log::HostLogLevel>,
        typ: Option<domain::native_host_log::HostLogType>,
    ) -> anyhow::Result<Vec<domain::native_host_log::NativeHostLogRow>> {
        self.host_log
            .lock()
            .await
            .list_logs(limit, offset, level, typ)
            .await
    }
    async fn count_logs(
        &mut self,
        level: Option<domain::native_host_log::HostLogLevel>,
        typ: Option<domain::native_host_log::HostLogType>,
    ) -> anyhow::Result<i64> {
        self.host_log.lock().await.count_logs(level, typ).await
    }
}

#[cfg(test)]
impl domain::repository::work_parent_packs::WorkParentPacksRepository for TestRepositories {
    async fn add(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
        parent_pack: domain::work_parent_pack::ParentPackKey,
    ) -> anyhow::Result<()> {
        self.work_parent_packs
            .lock()
            .await
            .add(work_id, parent_pack)
            .await
    }
    async fn exists(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
        parent_pack: domain::work_parent_pack::ParentPackKey,
    ) -> anyhow::Result<bool> {
        self.work_parent_packs
            .lock()
            .await
            .exists(work_id, parent_pack)
            .await
    }
    async fn find_parent_key(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
    ) -> anyhow::Result<Option<domain::work_parent_pack::ParentPackKey>> {
        self.work_parent_packs
            .lock()
            .await
            .find_parent_key(work_id)
            .await
    }
}

#[cfg(test)]
impl domain::repository::work_download_path::WorkDownloadPathRepository for TestRepositories {
    async fn add(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
        download_path: &str,
    ) -> anyhow::Result<()> {
        self.work_download_path
            .lock()
            .await
            .add(work_id, download_path)
            .await
    }
    async fn list_by_work(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
    ) -> anyhow::Result<Vec<domain::work_download_path::WorkDownloadPath>> {
        self.work_download_path
            .lock()
            .await
            .list_by_work(work_id)
            .await
    }
    async fn latest_by_work(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
    ) -> anyhow::Result<Option<domain::work_download_path::WorkDownloadPath>> {
        self.work_download_path
            .lock()
            .await
            .latest_by_work(work_id)
            .await
    }
}

#[cfg(test)]
impl domain::repository::work_lnk::WorkLnkRepository for TestRepositories {
    async fn find_by_id(
        &mut self,
        id: domain::Id<domain::repository::work_lnk::WorkLnk>,
    ) -> anyhow::Result<Option<domain::repository::work_lnk::WorkLnk>> {
        self.work_lnk.lock().await.find_by_id(id).await
    }
    async fn list_by_work_id(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
    ) -> anyhow::Result<Vec<domain::repository::work_lnk::WorkLnk>> {
        self.work_lnk.lock().await.list_by_work_id(work_id).await
    }
    async fn insert(
        &mut self,
        new_lnk: &domain::repository::work_lnk::NewWorkLnk,
    ) -> anyhow::Result<domain::Id<domain::repository::work_lnk::WorkLnk>> {
        self.work_lnk.lock().await.insert(new_lnk).await
    }
    async fn delete(
        &mut self,
        id: domain::Id<domain::repository::work_lnk::WorkLnk>,
    ) -> anyhow::Result<()> {
        self.work_lnk.lock().await.delete(id).await
    }
}

#[cfg(test)]
impl domain::repository::work_like::WorkLikeRepository for TestRepositories {
    async fn upsert(
        &mut self,
        like: &domain::works::NewWorkLike,
    ) -> anyhow::Result<domain::Id<domain::works::WorkLike>> {
        self.work_like.lock().await.upsert(like).await
    }
    async fn delete_by_work_id(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
    ) -> anyhow::Result<()> {
        self.work_like.lock().await.delete_by_work_id(work_id).await
    }
    async fn get_by_work_id(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
    ) -> anyhow::Result<Option<domain::works::WorkLike>> {
        self.work_like.lock().await.get_by_work_id(work_id).await
    }
    async fn update_like_at_by_work_id(
        &mut self,
        work_id: domain::StrId<domain::works::Work>,
        like_at: Option<chrono::DateTime<chrono::Local>>,
    ) -> anyhow::Result<()> {
        self.work_like
            .lock()
            .await
            .update_like_at_by_work_id(work_id, like_at)
            .await
    }
}

#[cfg(test)]
impl domain::work_link_pending_exe::WorkLinkPendingExeRepository for TestRepositories {
    async fn list_all(
        &mut self,
    ) -> anyhow::Result<Vec<domain::work_link_pending_exe::WorkLinkPendingExe>> {
        self.work_link_pending_exe.lock().await.list_all().await
    }
    async fn delete(
        &mut self,
        id: domain::Id<domain::work_link_pending_exe::WorkLinkPendingExe>,
    ) -> anyhow::Result<()> {
        self.work_link_pending_exe.lock().await.delete(id).await
    }
}

// Test RepositoryManager
#[cfg(test)]
pub struct TestRepositoryManager {
    repos: TestRepositories,
}

#[cfg(test)]
impl TestRepositoryManager {
    pub fn new(repos: TestRepositories) -> Self {
        Self { repos }
    }
}

#[cfg(test)]
impl domain::repository::manager::RepositoryManager<TestRepositories> for TestRepositoryManager {
    fn run<'a, T>(
        &'a self,
        f: impl FnOnce(TestRepositories) -> futures::future::BoxFuture<'a, anyhow::Result<T>>
            + Send
            + 'a,
    ) -> futures::future::BoxFuture<'a, anyhow::Result<T>> {
        futures::FutureExt::boxed(async move { f(self.repos.clone()).await })
    }
    fn run_in_transaction<'a, T>(
        &'a self,
        f: impl FnOnce(TestRepositories) -> futures::future::BoxFuture<'a, anyhow::Result<T>>
            + Send
            + 'a,
    ) -> futures::future::BoxFuture<'a, anyhow::Result<T>> {
        futures::FutureExt::boxed(async move { f(self.repos.clone()).await })
    }
}

#[cfg(test)]
impl TestRepositories {
    pub fn set_all_game_cache(
        &mut self,
        repo: domain::repository::all_game_cache::MockAllGameCacheRepository,
    ) {
        self.all_game_cache = Arc::new(tokio::sync::Mutex::new(repo));
    }
}
