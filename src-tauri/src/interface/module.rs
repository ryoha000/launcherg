use std::sync::Arc;

use tauri::AppHandle;

use crate::{
    domain::pubsub::PubSubService,
    domain::service::save_path_resolver::{DirsSavePathResolver},
    infrastructure::{
        explorerimpl::explorer::{Explorers, ExplorersExt},
        pubsubimpl::pubsub::{PubSub, PubSubExt},
        repositoryimpl::{
            driver::Db,
            repository::{Repositories, RepositoriesExt},
        },
        windowsimpl::windows::{Windows, WindowsExt},
        thumbnail::ThumbnailServiceImpl,
        icon::IconServiceImpl as TauriIconServiceImpl,
    },
    usecase::{
        all_game_cache::AllGameCacheUseCase, collection::CollectionUseCase,
        explored_cache::ExploredCacheUseCase, extension_manager::ExtensionManagerUseCase,
        file::FileUseCase, image::ImageUseCase, process::ProcessUseCase,
    },
};

pub struct Modules {
    repositories: Arc<Repositories>,
    collection_use_case: CollectionUseCase<Repositories>,
    explored_cache_use_case: ExploredCacheUseCase<Repositories>,
    extension_manager_use_case: ExtensionManagerUseCase<Repositories, PubSub>,
    file_use_case: FileUseCase<Explorers>,
    all_game_cache_use_case: AllGameCacheUseCase<Repositories>,
    process_use_case: ProcessUseCase<Windows>,
    image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl>,
    pubsub: PubSub,
}
pub trait ModulesExt {
    type Repositories: RepositoriesExt;
    type Explorers: ExplorersExt;
    type Windows: WindowsExt;
    type PubSub: PubSubExt + PubSubService;

    fn repositories(&self) -> &Self::Repositories;
    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories>;
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<Self::Repositories>;
    fn extension_manager_use_case(&self) -> &ExtensionManagerUseCase<Self::Repositories, Self::PubSub>;
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<Self::Repositories>;
    fn file_use_case(&self) -> &FileUseCase<Self::Explorers>;
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows>;
    fn image_use_case(&self) -> &ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl>;
    fn pubsub(&self) -> &Self::PubSub;
}

impl ModulesExt for Modules {
    type Repositories = Repositories;
    type Explorers = Explorers;
    type Windows = Windows;
    type PubSub = PubSub;

    fn repositories(&self) -> &Self::Repositories {
        &*self.repositories
    }
    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories> {
        &self.collection_use_case
    }
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<Self::Repositories> {
        &self.explored_cache_use_case
    }
    fn extension_manager_use_case(&self) -> &ExtensionManagerUseCase<Self::Repositories, Self::PubSub> {
        &self.extension_manager_use_case
    }
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<Self::Repositories> {
        &self.all_game_cache_use_case
    }
    fn file_use_case(&self) -> &FileUseCase<Self::Explorers> {
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
        let explorers = Arc::new(Explorers::new());
        let windows = Arc::new(Windows::new(Arc::new(handle.clone())));
        let pubsub = PubSub::new(Arc::new(handle.clone()));

        let collection_use_case = CollectionUseCase::new(repositories.clone(), Arc::new(DirsSavePathResolver::default()));
        let explored_cache_use_case = ExploredCacheUseCase::new(repositories.clone());
        let extension_manager_use_case = ExtensionManagerUseCase::new(repositories.clone(), pubsub.clone());
        let all_game_cache_use_case: AllGameCacheUseCase<Repositories> =
            AllGameCacheUseCase::new(repositories.clone());

        let file_use_case: FileUseCase<Explorers> = FileUseCase::new(explorers.clone());

        let process_use_case: ProcessUseCase<Windows> = ProcessUseCase::new(windows.clone());

        let thumbs = ThumbnailServiceImpl::new(Arc::new(DirsSavePathResolver::default()));
        let icons = TauriIconServiceImpl::new_from_app_handle(Arc::new(handle.clone()));
        let image_use_case: ImageUseCase<ThumbnailServiceImpl, TauriIconServiceImpl> = ImageUseCase::new(Arc::new(thumbs), Arc::new(icons), Arc::new(DirsSavePathResolver::default()));

        Self {
            repositories: repositories.clone(),
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
