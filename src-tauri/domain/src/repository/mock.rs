use std::sync::Arc;
use tauri::async_runtime::Mutex;

#[derive(Clone)]
pub struct TestRepositories {
    pub collection: Arc<Mutex<crate::repository::collection::MockCollectionRepository>>,
    pub explored_cache: Arc<Mutex<crate::repository::explored_cache::MockExploredCacheRepository>>,
    pub all_game_cache: Arc<Mutex<crate::repository::all_game_cache::MockAllGameCacheRepository>>,
    pub image_queue: Arc<Mutex<crate::repository::save_image_queue::MockImageSaveQueueRepository>>,
    pub host_log: Arc<Mutex<crate::repository::native_host_log::MockNativeHostLogRepository>>,
    pub work_omit: Arc<Mutex<crate::repository::work_omit::MockWorkOmitRepository>>,
    pub dmm_pack: Arc<Mutex<crate::repository::dmm_work_pack::MockDmmPackRepository>>,
    pub dmm_work: Arc<Mutex<crate::repository::works::MockDmmWorkRepository>>,
    pub dlsite_work: Arc<Mutex<crate::repository::works::MockDlsiteWorkRepository>>,
    pub work: Arc<Mutex<crate::repository::works::MockWorkRepository>>,
    pub work_parent_packs:
        Arc<Mutex<crate::repository::work_parent_packs::MockWorkParentPacksRepository>>,
    pub work_download_path:
        Arc<Mutex<crate::repository::work_download_path::MockWorkDownloadPathRepository>>,
    pub work_lnk: Arc<Mutex<crate::repository::work_lnk::MockWorkLnkRepository>>,
    pub erogamescape: Arc<Mutex<crate::repository::erogamescape::MockErogamescapeRepository>>,
}

impl Default for TestRepositories {
    fn default() -> Self {
        Self {
            collection: Arc::new(Mutex::new(Default::default())),
            explored_cache: Arc::new(Mutex::new(Default::default())),
            all_game_cache: Arc::new(Mutex::new(Default::default())),
            image_queue: Arc::new(Mutex::new(Default::default())),
            host_log: Arc::new(Mutex::new(Default::default())),
            work_omit: Arc::new(Mutex::new(Default::default())),
            dmm_pack: Arc::new(Mutex::new(Default::default())),
            dmm_work: Arc::new(Mutex::new(Default::default())),
            dlsite_work: Arc::new(Mutex::new(Default::default())),
            work: Arc::new(Mutex::new(Default::default())),
            work_parent_packs: Arc::new(Mutex::new(Default::default())),
            work_download_path: Arc::new(Mutex::new(Default::default())),
            work_lnk: Arc::new(Mutex::new(Default::default())),
            erogamescape: Arc::new(Mutex::new(Default::default())),
        }
    }
}

impl crate::repository::RepositoriesExt for TestRepositories {
    type WorkRepo = TestRepositories;
    type DmmWorkRepo = TestRepositories;
    type DlsiteWorkRepo = TestRepositories;
    type AllGameCacheRepo = TestRepositories;
    type ExploredCacheRepo = TestRepositories;
    type ImageQueueRepo = TestRepositories;
    type HostLogRepo = TestRepositories;
    type WorkOmitRepo = TestRepositories;
    type WorkParentPacksRepo = TestRepositories;
    type DmmPackRepo = TestRepositories;
    type CollectionRepo = TestRepositories;
    type ErogamescapeRepo = TestRepositories;
    type WorkDownloadPathRepo = TestRepositories;
    type WorkLnkRepo = TestRepositories;
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
    fn explored_cache(&self) -> Self::ExploredCacheRepo {
        self.clone()
    }
    fn image_queue(&self) -> Self::ImageQueueRepo {
        self.clone()
    }
    fn host_log(&self) -> Self::HostLogRepo {
        self.clone()
    }
    fn work_omit(&self) -> Self::WorkOmitRepo {
        self.clone()
    }
    fn work_parent_packs(&self) -> Self::WorkParentPacksRepo {
        self.clone()
    }
    fn dmm_pack(&self) -> Self::DmmPackRepo {
        self.clone()
    }
    fn collection(&self) -> Self::CollectionRepo {
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
}

impl crate::repository::works::WorkRepository for TestRepositories {
    async fn upsert(
        &mut self,
        new_work: &crate::works::NewWork,
    ) -> anyhow::Result<crate::Id<crate::works::Work>> {
        self.work.lock().await.upsert(new_work).await
    }
    async fn find_by_title(&mut self, title: &str) -> anyhow::Result<Option<crate::works::Work>> {
        self.work.lock().await.find_by_title(title).await
    }
    async fn list_all_details(&mut self) -> anyhow::Result<Vec<crate::works::WorkDetails>> {
        self.work.lock().await.list_all_details().await
    }
    async fn find_details_by_collection_element_id(
        &mut self,
        collection_element_id: crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<Option<crate::works::WorkDetails>> {
        self.work
            .lock()
            .await
            .find_details_by_collection_element_id(collection_element_id)
            .await
    }
    async fn find_work_ids_by_erogamescape_ids(
        &mut self,
        erogamescape_ids: &[i32],
    ) -> anyhow::Result<Vec<(i32, crate::Id<crate::works::Work>)>> {
        self.work
            .lock()
            .await
            .find_work_ids_by_erogamescape_ids(erogamescape_ids)
            .await
    }
    async fn upsert_info_by_erogamescape(
        &mut self,
        work_id: crate::Id<crate::works::Work>,
        erogamescape_id: i32,
        gamename_ruby: &str,
        brandname: &str,
        brandname_ruby: &str,
        sellday: &str,
        is_nukige: bool,
    ) -> anyhow::Result<()> {
        self.work
            .lock()
            .await
            .upsert_info_by_erogamescape(
                work_id,
                erogamescape_id,
                gamename_ruby,
                brandname,
                brandname_ruby,
                sellday,
                is_nukige,
            )
            .await
    }
}

impl crate::repository::erogamescape::ErogamescapeRepository for TestRepositories {
    async fn upsert_information(
        &mut self,
        info: &crate::erogamescape::NewErogamescapeInformation,
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

impl crate::repository::works::DmmWorkRepository for TestRepositories {
    async fn upsert(
        &mut self,
        new_work: &crate::works::NewDmmWork,
    ) -> anyhow::Result<crate::Id<crate::works::DmmWork>> {
        self.dmm_work.lock().await.upsert(new_work).await
    }
    async fn find_by_store_key(
        &mut self,
        store_id: &str,
        category: &str,
        subcategory: &str,
    ) -> anyhow::Result<Option<crate::works::DmmWork>> {
        self.dmm_work
            .lock()
            .await
            .find_by_store_key(store_id, category, subcategory)
            .await
    }
    async fn find_by_store_id(
        &mut self,
        store_id: &str,
    ) -> anyhow::Result<Option<crate::works::DmmWork>> {
        self.dmm_work.lock().await.find_by_store_id(store_id).await
    }
    async fn find_by_store_keys(
        &mut self,
        keys: &[(String, String, String)],
    ) -> anyhow::Result<Vec<crate::works::DmmWork>> {
        self.dmm_work.lock().await.find_by_store_keys(keys).await
    }
    async fn find_by_work_id(
        &mut self,
        work_id: crate::Id<crate::works::Work>,
    ) -> anyhow::Result<Option<crate::works::DmmWork>> {
        self.dmm_work.lock().await.find_by_work_id(work_id).await
    }
}

impl crate::repository::works::DlsiteWorkRepository for TestRepositories {
    async fn upsert(
        &mut self,
        new_work: &crate::works::NewDlsiteWork,
    ) -> anyhow::Result<crate::Id<crate::works::DlsiteWork>> {
        self.dlsite_work.lock().await.upsert(new_work).await
    }
    async fn find_by_store_key(
        &mut self,
        store_id: &str,
        category: &str,
    ) -> anyhow::Result<Option<crate::works::DlsiteWork>> {
        self.dlsite_work
            .lock()
            .await
            .find_by_store_key(store_id, category)
            .await
    }
    async fn find_by_store_id(
        &mut self,
        store_id: &str,
    ) -> anyhow::Result<Option<crate::works::DlsiteWork>> {
        self.dlsite_work
            .lock()
            .await
            .find_by_store_id(store_id)
            .await
    }
}

impl crate::repository::dmm_work_pack::DmmPackRepository for TestRepositories {
    async fn add(&mut self, work_id: crate::Id<crate::works::Work>) -> anyhow::Result<()> {
        self.dmm_pack.lock().await.add(work_id).await
    }
    async fn remove(&mut self, work_id: crate::Id<crate::works::Work>) -> anyhow::Result<()> {
        self.dmm_pack.lock().await.remove(work_id).await
    }
    async fn list(&mut self) -> anyhow::Result<Vec<crate::dmm_work_pack::DmmWorkPack>> {
        self.dmm_pack.lock().await.list().await
    }
    async fn exists(&mut self, work_id: crate::Id<crate::works::Work>) -> anyhow::Result<bool> {
        self.dmm_pack.lock().await.exists(work_id).await
    }
}

impl crate::repository::all_game_cache::AllGameCacheRepository for TestRepositories {
    async fn get_by_ids(
        &mut self,
        ids: Vec<i32>,
    ) -> anyhow::Result<Vec<crate::all_game_cache::AllGameCacheOneWithThumbnailUrl>> {
        self.all_game_cache.lock().await.get_by_ids(ids).await
    }
    async fn get_all(&mut self) -> anyhow::Result<crate::all_game_cache::AllGameCache> {
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
        cache: Vec<crate::all_game_cache::NewAllGameCacheOne>,
    ) -> anyhow::Result<()> {
        self.all_game_cache.lock().await.update(cache).await
    }
    async fn search_by_name(
        &mut self,
        name: &str,
    ) -> anyhow::Result<Vec<crate::all_game_cache::AllGameCacheOneWithThumbnailUrl>> {
        self.all_game_cache.lock().await.search_by_name(name).await
    }
}

impl crate::repository::explored_cache::ExploredCacheRepository for TestRepositories {
    async fn get_all(&mut self) -> anyhow::Result<crate::explored_cache::ExploredCache> {
        self.explored_cache.lock().await.get_all().await
    }
    async fn add(&mut self, adding: crate::explored_cache::ExploredCache) -> anyhow::Result<()> {
        self.explored_cache.lock().await.add(adding).await
    }
}

impl crate::repository::collection::CollectionRepository for TestRepositories {
    async fn get_all_elements(
        &mut self,
    ) -> anyhow::Result<Vec<crate::collection::CollectionElement>> {
        self.collection.lock().await.get_all_elements().await
    }
    async fn get_element_by_element_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<Option<crate::collection::CollectionElement>> {
        self.collection
            .lock()
            .await
            .get_element_by_element_id(id)
            .await
    }
    async fn upsert_collection_element(
        &mut self,
        new_element: &crate::collection::NewCollectionElement,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .upsert_collection_element(new_element)
            .await
    }
    async fn update_collection_element_gamename_by_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
        gamename: &str,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .update_collection_element_gamename_by_id(id, gamename)
            .await
    }
    async fn delete_collection_element(
        &mut self,
        element_id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .delete_collection_element(element_id)
            .await
    }
    async fn upsert_collection_element_paths(
        &mut self,
        paths: &crate::collection::NewCollectionElementPaths,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .upsert_collection_element_paths(paths)
            .await
    }
    async fn get_element_paths_by_element_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<Option<crate::collection::CollectionElementPaths>> {
        self.collection
            .lock()
            .await
            .get_element_paths_by_element_id(id)
            .await
    }
    async fn upsert_collection_element_install(
        &mut self,
        install: &crate::collection::NewCollectionElementInstall,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .upsert_collection_element_install(install)
            .await
    }
    async fn get_element_install_by_element_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<Option<crate::collection::CollectionElementInstall>> {
        self.collection
            .lock()
            .await
            .get_element_install_by_element_id(id)
            .await
    }
    async fn upsert_collection_element_play(
        &mut self,
        play: &crate::collection::NewCollectionElementPlay,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .upsert_collection_element_play(play)
            .await
    }
    async fn get_element_play_by_element_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<Option<crate::collection::CollectionElementPlay>> {
        self.collection
            .lock()
            .await
            .get_element_play_by_element_id(id)
            .await
    }
    async fn update_element_last_play_at_by_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
        last_play_at: chrono::DateTime<chrono::Local>,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .update_element_last_play_at_by_id(id, last_play_at)
            .await
    }
    async fn upsert_collection_element_like(
        &mut self,
        like: &crate::collection::NewCollectionElementLike,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .upsert_collection_element_like(like)
            .await
    }
    async fn delete_collection_element_like_by_element_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .delete_collection_element_like_by_element_id(id)
            .await
    }
    async fn get_element_like_by_element_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<Option<crate::collection::CollectionElementLike>> {
        self.collection
            .lock()
            .await
            .get_element_like_by_element_id(id)
            .await
    }
    async fn update_element_like_at_by_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
        like_at: Option<chrono::DateTime<chrono::Local>>,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .update_element_like_at_by_id(id, like_at)
            .await
    }
    async fn upsert_collection_element_thumbnail(
        &mut self,
        thumbnail: &crate::collection::NewCollectionElementThumbnail,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .upsert_collection_element_thumbnail(thumbnail)
            .await
    }
    async fn get_element_thumbnail_by_element_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<Option<crate::collection::CollectionElementThumbnail>> {
        self.collection
            .lock()
            .await
            .get_element_thumbnail_by_element_id(id)
            .await
    }
    async fn upsert_collection_element_thumbnail_size(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
        width: i32,
        height: i32,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .upsert_collection_element_thumbnail_size(id, width, height)
            .await
    }
    async fn get_null_thumbnail_size_element_ids(
        &mut self,
    ) -> anyhow::Result<Vec<crate::Id<crate::collection::CollectionElement>>> {
        self.collection
            .lock()
            .await
            .get_null_thumbnail_size_element_ids()
            .await
    }
    async fn get_collection_id_by_erogamescape_id(
        &mut self,
        erogamescape_id: i32,
    ) -> anyhow::Result<Option<crate::Id<crate::collection::CollectionElement>>> {
        self.collection
            .lock()
            .await
            .get_collection_id_by_erogamescape_id(erogamescape_id)
            .await
    }
    async fn get_collection_ids_by_erogamescape_ids(
        &mut self,
        erogamescape_ids: &[i32],
    ) -> anyhow::Result<Vec<(i32, crate::Id<crate::collection::CollectionElement>)>> {
        self.collection
            .lock()
            .await
            .get_collection_ids_by_erogamescape_ids(erogamescape_ids)
            .await
    }
    async fn get_collection_ids_by_work_ids(
        &mut self,
        work_ids: &[crate::Id<crate::works::Work>],
    ) -> anyhow::Result<
        Vec<(
            crate::Id<crate::works::Work>,
            crate::Id<crate::collection::CollectionElement>,
        )>,
    > {
        self.collection
            .lock()
            .await
            .get_collection_ids_by_work_ids(work_ids)
            .await
    }
    async fn upsert_work_mapping(
        &mut self,
        collection_element_id: &crate::Id<crate::collection::CollectionElement>,
        work_id: crate::Id<crate::works::Work>,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .upsert_work_mapping(collection_element_id, work_id)
            .await
    }
    async fn insert_work_mapping(
        &mut self,
        collection_element_id: &crate::Id<crate::collection::CollectionElement>,
        work_id: crate::Id<crate::works::Work>,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .insert_work_mapping(collection_element_id, work_id)
            .await
    }
    async fn get_work_ids_by_collection_ids(
        &mut self,
        collection_element_ids: &[crate::Id<crate::collection::CollectionElement>],
    ) -> anyhow::Result<
        Vec<(
            crate::Id<crate::collection::CollectionElement>,
            crate::Id<crate::works::Work>,
        )>,
    > {
        self.collection
            .lock()
            .await
            .get_work_ids_by_collection_ids(collection_element_ids)
            .await
    }
    async fn get_element_erogamescape_by_element_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<Option<crate::collection::CollectionElementErogamescape>> {
        self.collection
            .lock()
            .await
            .get_element_erogamescape_by_element_id(id)
            .await
    }
    async fn upsert_erogamescape_map(
        &mut self,
        collection_element_id: &crate::Id<crate::collection::CollectionElement>,
        erogamescape_id: i32,
    ) -> anyhow::Result<()> {
        self.collection
            .lock()
            .await
            .upsert_erogamescape_map(collection_element_id, erogamescape_id)
            .await
    }
    async fn allocate_new_collection_element_id(
        &mut self,
        gamename: &str,
    ) -> anyhow::Result<crate::Id<crate::collection::CollectionElement>> {
        self.collection
            .lock()
            .await
            .allocate_new_collection_element_id(gamename)
            .await
    }
    async fn get_erogamescape_id_by_collection_id(
        &mut self,
        id: &crate::Id<crate::collection::CollectionElement>,
    ) -> anyhow::Result<Option<i32>> {
        self.collection
            .lock()
            .await
            .get_erogamescape_id_by_collection_id(id)
            .await
    }
}

impl crate::repository::save_image_queue::ImageSaveQueueRepository for TestRepositories {
    async fn enqueue(
        &mut self,
        image_url: &str,
        src_type: crate::save_image_queue::ImageSrcType,
        dst_path: &str,
        preprocess: crate::save_image_queue::ImagePreprocess,
    ) -> anyhow::Result<crate::Id<crate::save_image_queue::ImageSaveQueueRow>> {
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
    ) -> anyhow::Result<Vec<crate::save_image_queue::ImageSaveQueueRow>> {
        self.image_queue.lock().await.list(unfinished, limit).await
    }
    async fn mark_finished(
        &mut self,
        id: crate::Id<crate::save_image_queue::ImageSaveQueueRow>,
    ) -> anyhow::Result<()> {
        self.image_queue.lock().await.mark_finished(id).await
    }
    async fn mark_failed(
        &mut self,
        id: crate::Id<crate::save_image_queue::ImageSaveQueueRow>,
        error: &str,
    ) -> anyhow::Result<()> {
        self.image_queue.lock().await.mark_failed(id, error).await
    }
}

impl crate::repository::native_host_log::NativeHostLogRepository for TestRepositories {
    async fn insert_log(
        &mut self,
        level: crate::native_host_log::HostLogLevel,
        typ: crate::native_host_log::HostLogType,
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
        level: Option<crate::native_host_log::HostLogLevel>,
        typ: Option<crate::native_host_log::HostLogType>,
    ) -> anyhow::Result<Vec<crate::native_host_log::NativeHostLogRow>> {
        self.host_log
            .lock()
            .await
            .list_logs(limit, offset, level, typ)
            .await
    }
    async fn count_logs(
        &mut self,
        level: Option<crate::native_host_log::HostLogLevel>,
        typ: Option<crate::native_host_log::HostLogType>,
    ) -> anyhow::Result<i64> {
        self.host_log.lock().await.count_logs(level, typ).await
    }
}

impl crate::repository::work_omit::WorkOmitRepository for TestRepositories {
    async fn add(&mut self, work_id: crate::Id<crate::works::Work>) -> anyhow::Result<()> {
        self.work_omit.lock().await.add(work_id).await
    }
    async fn remove(&mut self, work_id: crate::Id<crate::works::Work>) -> anyhow::Result<()> {
        self.work_omit.lock().await.remove(work_id).await
    }
    async fn list(&mut self) -> anyhow::Result<Vec<crate::work_omit::WorkOmit>> {
        self.work_omit.lock().await.list().await
    }
    async fn exists(&mut self, work_id: crate::Id<crate::works::Work>) -> anyhow::Result<bool> {
        self.work_omit.lock().await.exists(work_id).await
    }
}

impl crate::repository::work_parent_packs::WorkParentPacksRepository for TestRepositories {
    async fn add(
        &mut self,
        work_id: crate::Id<crate::works::Work>,
        parent_pack_work_id: crate::Id<crate::works::Work>,
    ) -> anyhow::Result<()> {
        self.work_parent_packs
            .lock()
            .await
            .add(work_id, parent_pack_work_id)
            .await
    }
    async fn exists(
        &mut self,
        work_id: crate::Id<crate::works::Work>,
        parent_pack_work_id: crate::Id<crate::works::Work>,
    ) -> anyhow::Result<bool> {
        self.work_parent_packs
            .lock()
            .await
            .exists(work_id, parent_pack_work_id)
            .await
    }
    async fn find_parent_id(
        &mut self,
        work_id: crate::Id<crate::works::Work>,
    ) -> anyhow::Result<Option<crate::Id<crate::works::Work>>> {
        self.work_parent_packs
            .lock()
            .await
            .find_parent_id(work_id)
            .await
    }
}

impl crate::repository::work_download_path::WorkDownloadPathRepository for TestRepositories {
    async fn add(
        &mut self,
        work_id: crate::Id<crate::works::Work>,
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
        work_id: crate::Id<crate::works::Work>,
    ) -> anyhow::Result<Vec<crate::work_download_path::WorkDownloadPath>> {
        self.work_download_path
            .lock()
            .await
            .list_by_work(work_id)
            .await
    }
    async fn latest_by_work(
        &mut self,
        work_id: crate::Id<crate::works::Work>,
    ) -> anyhow::Result<Option<crate::work_download_path::WorkDownloadPath>> {
        self.work_download_path
            .lock()
            .await
            .latest_by_work(work_id)
            .await
    }
}

impl crate::repository::work_lnk::WorkLnkRepository for TestRepositories {
    async fn find_by_id(
        &mut self,
        id: crate::Id<crate::repository::work_lnk::WorkLnk>,
    ) -> anyhow::Result<Option<crate::repository::work_lnk::WorkLnk>> {
        self.work_lnk.lock().await.find_by_id(id).await
    }
    async fn list_by_work_id(
        &mut self,
        work_id: crate::Id<crate::works::Work>,
    ) -> anyhow::Result<Vec<crate::repository::work_lnk::WorkLnk>> {
        self.work_lnk.lock().await.list_by_work_id(work_id).await
    }
    async fn insert(
        &mut self,
        new_lnk: &crate::repository::work_lnk::NewWorkLnk,
    ) -> anyhow::Result<crate::Id<crate::repository::work_lnk::WorkLnk>> {
        self.work_lnk.lock().await.insert(new_lnk).await
    }
    async fn delete(
        &mut self,
        id: crate::Id<crate::repository::work_lnk::WorkLnk>,
    ) -> anyhow::Result<()> {
        self.work_lnk.lock().await.delete(id).await
    }
}

// Test RepositoryManager

pub struct TestRepositoryManager {
    repos: TestRepositories,
}

impl TestRepositoryManager {
    pub fn new(repos: TestRepositories) -> Self {
        Self { repos }
    }
}

impl crate::repository::manager::RepositoryManager<TestRepositories> for TestRepositoryManager {
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
