use derive_new::new;
use std::marker::PhantomData;

use crate::domain::{collection::Collection, repository::collection::CollectionRepository};

use super::driver::Db;

#[derive(new)]
pub struct RepositoryImpl<T> {
    pub pool: Db,
    _marker: PhantomData<T>,
}

pub struct Repositories {
    collection_repository: RepositoryImpl<Collection>,
}
pub trait RepositoriesExt {
    type CollectionRepo: CollectionRepository;

    fn collection_repository(&self) -> &Self::CollectionRepo;
}

impl RepositoriesExt for Repositories {
    type CollectionRepo = RepositoryImpl<Collection>;

    fn collection_repository(&self) -> &Self::CollectionRepo {
        &self.collection_repository
    }
}

impl Repositories {
    pub fn new(db: Db) -> Self {
        let collection_repository = RepositoryImpl::new(db.clone());

        Self {
            collection_repository,
        }
    }
}
