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
            sqliterepository::{SqliteRepository, RepositoryExecutor},
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
    repositories: Arc<tokio::sync::Mutex<SqliteRepository<'static>>>,
    collection_use_case: CollectionUseCase<SqliteRepository<'static>, ThumbnailServiceImpl>,
    explored_cache_use_case: ExploredCacheUseCase<SqliteRepository<'static>>,
    extension_manager_use_case: ExtensionManagerUseCase<PubSub, NativeMessagingHostClientFactoryImpl>,
    file_use_case: FileUseCase,
    all_game_cache_use_case: AllGameCacheUseCase<SqliteRepository<'static>>,
    process_use_case: ProcessUseCase<Windows>,
    image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl>,
    work_omit_use_case: WorkOmitUseCase<SqliteRepository<'static>>,
    host_log_use_case: HostLogUseCase<SqliteRepository<'static>>,
    dmm_pack_use_case: DmmPackUseCase<SqliteRepository<'static>>,
    work_use_case: WorkUseCase<SqliteRepository<'static>>,
    pubsub: PubSub,
}
pub trait ModulesExt {
    type Repositories: RepositoriesExt;
    type Windows: WindowsExt;
    type PubSub: PubSubExt + PubSubService;

    fn repositories(&self) -> &Self::Repositories;
    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories, ThumbnailServiceImpl>;
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<Self::Repositories>;
    fn extension_manager_use_case(&self) -> &ExtensionManagerUseCase<Self::PubSub, NativeMessagingHostClientFactoryImpl>;
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<Self::Repositories>;
    fn file_use_case(&self) -> &FileUseCase;
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows>;
    fn image_use_case(&self) -> &ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl>;
    fn work_omit_use_case(&self) -> &WorkOmitUseCase<Self::Repositories>;
    fn host_log_use_case(&self) -> &HostLogUseCase<Self::Repositories>;
    fn dmm_pack_use_case(&self) -> &DmmPackUseCase<Self::Repositories>;
    fn work_use_case(&self) -> &WorkUseCase<Self::Repositories>;
    fn pubsub(&self) -> &Self::PubSub;
}

impl ModulesExt for Modules {
    type Repositories = SqliteRepository<'static>;
    type Windows = Windows;
    type PubSub = PubSub;

    fn repositories(&self) -> &Self::Repositories {
        // 注意: 実運用では &MutexGuard を返す設計にしないと参照がずれる。ここは未使用のため未実装にしておくか、インターフェースを見直す。
        unimplemented!("Modules::repositories() is no longer used directly with Mutex-wrapped repos");
    }
    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories, ThumbnailServiceImpl> {
        &self.collection_use_case
    }
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<Self::Repositories> {
        &self.explored_cache_use_case
    }
    fn extension_manager_use_case(&self) -> &ExtensionManagerUseCase<Self::PubSub, NativeMessagingHostClientFactoryImpl> {
        &self.extension_manager_use_case
    }
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<Self::Repositories> {
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
    fn work_omit_use_case(&self) -> &WorkOmitUseCase<Self::Repositories> { &self.work_omit_use_case }
    fn host_log_use_case(&self) -> &HostLogUseCase<Self::Repositories> { &self.host_log_use_case }
    fn dmm_pack_use_case(&self) -> &DmmPackUseCase<Self::Repositories> { &self.dmm_pack_use_case }
    fn work_use_case(&self) -> &WorkUseCase<Self::Repositories> { &self.work_use_case }
    fn pubsub(&self) -> &Self::PubSub {
        &self.pubsub
    }
}

impl Modules {
    pub async fn new(handle: &AppHandle) -> Self {
        let db = Db::new(handle).await;

        let repo = SqliteRepository::new(RepositoryExecutor::OwnedPool(db.pool_arc()));
        let repositories = Arc::new(tokio::sync::Mutex::new(repo));
        let windows = Arc::new(Windows::new(Arc::new(handle.clone())));
        let pubsub = PubSub::new(Arc::new(handle.clone()));
        let resolver = Arc::new(DirsSavePathResolver::default());

        let thumbs = Arc::new(ThumbnailServiceImpl::new(resolver.clone()));
        let icons = TauriIconServiceImpl::new_from_app_handle(Arc::new(handle.clone()));

        let collection_use_case = CollectionUseCase::new(repositories.clone(), resolver.clone(), thumbs.clone());
        let explored_cache_use_case = ExploredCacheUseCase::new(repositories.clone());
        let extension_manager_use_case = ExtensionManagerUseCase::new(pubsub.clone(), Arc::new(NativeMessagingHostClientFactoryImpl));
        let all_game_cache_use_case: AllGameCacheUseCase<SqliteRepository> =
            AllGameCacheUseCase::new(repositories.clone());

        let file_use_case: FileUseCase = FileUseCase::new(resolver.clone());

        let process_use_case: ProcessUseCase<Windows> = ProcessUseCase::new(windows.clone());

        let image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl> = ImageUseCase::new(thumbs.clone(), Arc::new(icons), resolver.clone());
        let work_omit_use_case: WorkOmitUseCase<SqliteRepository> = WorkOmitUseCase::new(repositories.clone());
        let host_log_use_case: HostLogUseCase<SqliteRepository> = HostLogUseCase::new(repositories.clone());
        let dmm_pack_use_case: DmmPackUseCase<SqliteRepository> = DmmPackUseCase::new(repositories.clone());
        let work_use_case: WorkUseCase<SqliteRepository> = WorkUseCase::new(repositories.clone());

        Self {
            repositories,
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
