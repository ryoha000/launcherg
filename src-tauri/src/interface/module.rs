use std::sync::Arc;

use crate::{
    infrastructure::repositoryimpl::{
        driver::Db,
        migration::migration,
        repository::{Repositories, RepositoriesExt},
    },
    usecase::collection::CollectionUseCase,
};

pub struct Modules {
    collection_use_case: CollectionUseCase<Repositories>,
}
pub trait ModulesExt {
    type Repositories: RepositoriesExt;

    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories>;
}

impl ModulesExt for Modules {
    type Repositories = Repositories;

    fn collection_use_case(&self) -> &CollectionUseCase<Self::Repositories> {
        &self.collection_use_case
    }
}

impl Modules {
    pub async fn new() -> Self {
        migration().await;
        let db = Db::new().await;

        let repositories = Arc::new(Repositories::new(db.clone()));

        let collection_use_case = CollectionUseCase::new(repositories);

        Self {
            collection_use_case,
        }
    }
}
