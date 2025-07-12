use std::sync::Arc;

use tauri::AppHandle;

use crate::{
    infrastructure::{
        explorerimpl::explorer::{Explorers, ExplorersExt},
        repositoryimpl::{
            driver::Db,
            repository::{Repositories, RepositoriesExt},
        },
        windowsimpl::windows::{Windows, WindowsExt},
    },
    usecase::{
        all_game_cache::AllGameCacheUseCase, collection::CollectionUseCase,
        explored_cache::ExploredCacheUseCase, file::FileUseCase, network::NetworkUseCase,
        process::ProcessUseCase,
    },
};

pub struct Modules {
    collection_use_case: CollectionUseCase<Repositories>,
    explored_cache_use_case: ExploredCacheUseCase<Repositories>,
    network_use_case: NetworkUseCase<Explorers>,
    file_use_case: FileUseCase<Explorers>,
    all_game_cache_use_case: AllGameCacheUseCase<Repositories>,
    process_use_case: ProcessUseCase<Windows>,
}
pub trait ModulesExt {
    type Repositories: RepositoriesExt;
    type Explorers: ExplorersExt;
    type Windows: WindowsExt;

    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories>;
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<Self::Repositories>;
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<Self::Repositories>;
    fn network_use_case(&self) -> &NetworkUseCase<Self::Explorers>;
    fn file_use_case(&self) -> &FileUseCase<Self::Explorers>;
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows>;
}

impl ModulesExt for Modules {
    type Repositories = Repositories;
    type Explorers = Explorers;
    type Windows = Windows;

    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories> {
        &self.collection_use_case
    }
    fn explored_cache_use_case(&self) -> &ExploredCacheUseCase<Self::Repositories> {
        &self.explored_cache_use_case
    }
    fn all_game_cache_use_case(&self) -> &AllGameCacheUseCase<Self::Repositories> {
        &self.all_game_cache_use_case
    }
    fn network_use_case(&self) -> &NetworkUseCase<Self::Explorers> {
        &self.network_use_case
    }
    fn file_use_case(&self) -> &FileUseCase<Self::Explorers> {
        &self.file_use_case
    }
    fn process_use_case(&self) -> &ProcessUseCase<Self::Windows> {
        &self.process_use_case
    }
}

impl Modules {
    pub async fn new(handle: &AppHandle) -> Self {
        let db = Db::new(handle).await;

        let repositories = Arc::new(Repositories::new(db.clone()));
        let explorers = Arc::new(Explorers::new());
        let windows = Arc::new(Windows::new());

        let collection_use_case = CollectionUseCase::new(repositories.clone());
        let explored_cache_use_case = ExploredCacheUseCase::new(repositories.clone());
        let all_game_cache_use_case: AllGameCacheUseCase<Repositories> =
            AllGameCacheUseCase::new(repositories.clone());

        let network_use_case: NetworkUseCase<Explorers> = NetworkUseCase::new(explorers.clone());
        let file_use_case: FileUseCase<Explorers> = FileUseCase::new(explorers.clone());

        let process_use_case: ProcessUseCase<Windows> = ProcessUseCase::new(windows.clone());

        Self {
            collection_use_case,
            explored_cache_use_case,
            all_game_cache_use_case,
            network_use_case,
            file_use_case,
            process_use_case,
        }
    }
}
