use std::sync::Arc;

use tauri::AppHandle;

use crate::{
    domain::{pubsub::PubSubService, repository::RepositoriesExt},
    domain::service::save_path_resolver::{DirsSavePathResolver},
    domain::windows::WindowsExt,
    infrastructure::{
        pubsubimpl::pubsub::{PubSub, PubSubExt},
        sqliterepository::{
            driver::Db,
            sqliterepository::{SqliteRepositories, SqliteRepositoryManager},
        },
        windowsimpl::windows::Windows,
        thumbnail::ThumbnailServiceImpl,
        icon::IconServiceImpl as TauriIconServiceImpl,
        native_messaging::NativeMessagingHostClientFactoryImpl,
    },
    usecase::{
        all_game_cache::AllGameCacheUseCase, collection::CollectionUseCase,
        explored_cache::ExploredCacheUseCase, extension_manager::ExtensionManagerUseCase,
        file::FileUseCase, image::ImageUseCase, process::ProcessUseCase,
        work_omit::WorkOmitUseCase,
        host_log::HostLogUseCase,
        dmm_pack::DmmPackUseCase,
        work::WorkUseCase,
    },
};

pub struct Modules {
    repositories: Arc<SqliteRepositoryManager>,
    collection_use_case: CollectionUseCase<SqliteRepositoryManager, SqliteRepositories, ThumbnailServiceImpl>,
    explored_cache_use_case: ExploredCacheUseCase<SqliteRepositoryManager, SqliteRepositories>,
    extension_manager_use_case: ExtensionManagerUseCase<PubSub, NativeMessagingHostClientFactoryImpl>,
    file_use_case: FileUseCase,
    all_game_cache_use_case: AllGameCacheUseCase<SqliteRepositoryManager, SqliteRepositories>,
    process_use_case: ProcessUseCase<Windows>,
    image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl>,
    work_omit_use_case: WorkOmitUseCase<SqliteRepositoryManager, SqliteRepositories>,
    host_log_use_case: HostLogUseCase<SqliteRepositoryManager, SqliteRepositories>,
    dmm_pack_use_case: DmmPackUseCase<SqliteRepositoryManager, SqliteRepositories>,
    work_use_case: WorkUseCase<SqliteRepositoryManager, SqliteRepositories>,
    pubsub: PubSub,
}
pub trait ModulesExt {
    type Repositories: RepositoriesExt;
    type Windows: WindowsExt;
    type PubSub: PubSubExt + PubSubService;

    fn repositories(&self) -> &Self::Repositories;
    fn collection_use_case(&self) -> &CollectionUseCase<SqliteRepositoryManager, SqliteRepositories, ThumbnailServiceImpl>;
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn extension_manager_use_case(&self) -> &ExtensionManagerUseCase<Self::PubSub, NativeMessagingHostClientFactoryImpl>;
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn file_use_case(&self) -> &FileUseCase;
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows>;
    fn image_use_case(&self) -> &ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl>;
    fn work_omit_use_case(&self) -> &WorkOmitUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn host_log_use_case(&self) -> &HostLogUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn dmm_pack_use_case(&self) -> &DmmPackUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn work_use_case(&self) -> &WorkUseCase<SqliteRepositoryManager, SqliteRepositories>;
    fn pubsub(&self) -> &Self::PubSub;
}

impl ModulesExt for Modules {
    type Repositories = SqliteRepositories;
    type Windows = Windows;
    type PubSub = PubSub;

    fn repositories(&self) -> &Self::Repositories { unimplemented!() }
    fn collection_use_case(&self) -> &CollectionUseCase<SqliteRepositoryManager, SqliteRepositories, ThumbnailServiceImpl> {
        &self.collection_use_case
    }
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<SqliteRepositoryManager, SqliteRepositories> {
        &self.explored_cache_use_case
    }
    fn extension_manager_use_case(&self) -> &ExtensionManagerUseCase<Self::PubSub, NativeMessagingHostClientFactoryImpl> {
        &self.extension_manager_use_case
    }
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<SqliteRepositoryManager, SqliteRepositories> {
        &self.all_game_cache_use_case
    }
    fn file_use_case(&self) -> &FileUseCase {
        &self.file_use_case
    }
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows> {
        &self.process_use_case
    }
    fn image_use_case(&self) -> &ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl> {
        &self.image_use_case
    }
    fn work_omit_use_case(&self) -> &WorkOmitUseCase<SqliteRepositoryManager, SqliteRepositories> { &self.work_omit_use_case }
    fn host_log_use_case(&self) -> &HostLogUseCase<SqliteRepositoryManager, SqliteRepositories> { &self.host_log_use_case }
    fn dmm_pack_use_case(&self) -> &DmmPackUseCase<SqliteRepositoryManager, SqliteRepositories> { &self.dmm_pack_use_case }
    fn work_use_case(&self) -> &WorkUseCase<SqliteRepositoryManager, SqliteRepositories> { &self.work_use_case }
    fn pubsub(&self) -> &Self::PubSub {
        &self.pubsub
    }
}

impl Modules {
    pub async fn new(handle: &AppHandle) -> Self {
        let db = Db::new(handle).await;

        let repo_manager = Arc::new(SqliteRepositoryManager::new(db.pool_arc()));
        let windows = Arc::new(Windows::new(Arc::new(handle.clone())));
        let pubsub = PubSub::new(Arc::new(handle.clone()));
        let resolver = Arc::new(DirsSavePathResolver::default());

        let thumbs = Arc::new(ThumbnailServiceImpl::new(resolver.clone()));
        let icons = TauriIconServiceImpl::new_from_app_handle(Arc::new(handle.clone()));

        let collection_use_case = CollectionUseCase::new(repo_manager.clone(), resolver.clone(), thumbs.clone());
        let explored_cache_use_case = ExploredCacheUseCase::new(repo_manager.clone());
        let extension_manager_use_case = ExtensionManagerUseCase::new(pubsub.clone(), Arc::new(NativeMessagingHostClientFactoryImpl));
        let all_game_cache_use_case: AllGameCacheUseCase<SqliteRepositoryManager, SqliteRepositories> =
            AllGameCacheUseCase::new(repo_manager.clone());

        let file_use_case: FileUseCase = FileUseCase::new(resolver.clone());

        let process_use_case: ProcessUseCase<Windows> = ProcessUseCase::new(windows.clone());

        let image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl> = ImageUseCase::new(thumbs.clone(), Arc::new(icons), resolver.clone());
        let work_omit_use_case: WorkOmitUseCase<SqliteRepositoryManager, SqliteRepositories> = WorkOmitUseCase::new(repo_manager.clone());
        let host_log_use_case: HostLogUseCase<SqliteRepositoryManager, SqliteRepositories> = HostLogUseCase::new(repo_manager.clone());
        let dmm_pack_use_case: DmmPackUseCase<SqliteRepositoryManager, SqliteRepositories> = DmmPackUseCase::new(repo_manager.clone());
        let work_use_case: WorkUseCase<SqliteRepositoryManager, SqliteRepositories> = WorkUseCase::new(repo_manager.clone());

        Self {
            repositories: repo_manager,
            collection_use_case,
            explored_cache_use_case,
            extension_manager_use_case,
            all_game_cache_use_case,
            file_use_case,
            process_use_case,
            image_use_case,
            work_omit_use_case,
            host_log_use_case,
            dmm_pack_use_case,
            work_use_case,
            pubsub,
        }
    }
}
