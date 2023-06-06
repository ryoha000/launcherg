use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("コレクションはすでに存在しています")]
    CollectionIsAlreadyExist,
}
