use std::sync::Arc;

use tauri::AppHandle;

use crate::{
    domain::{pubsub::PubSubService, repository::RepositoriesExt},
    domain::service::save_path_resolver::{DirsSavePathResolver},
    domain::windows::WindowsExt,
    infrastructure::{
        pubsubimpl::pubsub::{PubSub, PubSubExt},
        repositoryimpl::{
            driver::Db,
            repository::Repositories,
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
    },
};

pub struct Modules {
    repositories: Arc<Repositories>,
    collection_use_case: CollectionUseCase<Repositories, ThumbnailServiceImpl>,
    explored_cache_use_case: ExploredCacheUseCase<Repositories>,
    extension_manager_use_case: ExtensionManagerUseCase<PubSub, NativeMessagingHostClientFactoryImpl>,
    file_use_case: FileUseCase,
    all_game_cache_use_case: AllGameCacheUseCase<Repositories>,
    process_use_case: ProcessUseCase<Windows>,
    image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl>,
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
    fn pubsub(&self) -> &Self::PubSub;
}

impl ModulesExt for Modules {
    type Repositories = Repositories;
    type Windows = Windows;
    type PubSub = PubSub;

    fn repositories(&self) -> &Self::Repositories {
        &*self.repositories
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
    fn pubsub(&self) -> &Self::PubSub {
        &self.pubsub
    }
}

impl Modules {
    pub async fn new(handle: &AppHandle) -> Self {
        let db = Db::new(handle).await;

        let repositories = Arc::new(Repositories::new(db.clone()));
        let windows = Arc::new(Windows::new(Arc::new(handle.clone())));
        let pubsub = PubSub::new(Arc::new(handle.clone()));
        let resolver = Arc::new(DirsSavePathResolver::default());

        let thumbs = Arc::new(ThumbnailServiceImpl::new(resolver.clone()));
        let icons = TauriIconServiceImpl::new_from_app_handle(Arc::new(handle.clone()));

        let collection_use_case = CollectionUseCase::new(repositories.clone(), resolver.clone(), thumbs.clone());
        let explored_cache_use_case = ExploredCacheUseCase::new(repositories.clone());
        let extension_manager_use_case = ExtensionManagerUseCase::new(pubsub.clone(), Arc::new(NativeMessagingHostClientFactoryImpl));
        let all_game_cache_use_case: AllGameCacheUseCase<Repositories> =
            AllGameCacheUseCase::new(repositories.clone());

        let file_use_case: FileUseCase = FileUseCase::new(resolver.clone());

        let process_use_case: ProcessUseCase<Windows> = ProcessUseCase::new(windows.clone());

        let image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl> = ImageUseCase::new(thumbs.clone(), Arc::new(icons), resolver.clone());

        Self {
            repositories,
            collection_use_case,
            explored_cache_use_case,
            extension_manager_use_case,
            all_game_cache_use_case,
            file_use_case,
            process_use_case,
            image_use_case,
            pubsub,
        }
    }
}
