use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("collection is already exist")]
    CollectionIsAlreadyExist,
}
