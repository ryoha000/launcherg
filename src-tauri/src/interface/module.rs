use std::sync::Arc;

use crate::{
    infrastructure::{
        explorerimpl::explorer::{Explorers, ExplorersExt},
        repositoryimpl::{
            driver::Db,
            migration::migration,
            repository::{Repositories, RepositoriesExt},
        },
    },
    usecase::{collection::CollectionUseCase, network::NetworkUseCase},
};

pub struct Modules {
    collection_use_case: CollectionUseCase<Repositories>,
    network_use_case: NetworkUseCase<Explorers>,
}
pub trait ModulesExt {
    type Repositories: RepositoriesExt;
    type Explorers: ExplorersExt;

    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories>;
    fn network_use_case(&self) -> &NetworkUseCase<Self::Explorers>;
}

impl ModulesExt for Modules {
    type Repositories = Repositories;
    type Explorers = Explorers;

    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories> {
        &self.collection_use_case
    }
    fn network_use_case(&self) -> &NetworkUseCase<Self::Explorers> {
        &self.network_use_case
    }
}

impl Modules {
    pub async fn new() -> Self {
        migration().await;
        let db = Db::new().await;

        let repositories = Arc::new(Repositories::new(db.clone()));
        let explorers = Arc::new(Explorers::new());

        let collection_use_case = CollectionUseCase::new(repositories);

        let network_use_case: NetworkUseCase<Explorers> = NetworkUseCase::new(explorers);

        Self {
            collection_use_case,
            network_use_case,
        }
    }
}
