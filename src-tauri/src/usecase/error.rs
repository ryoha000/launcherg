use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("コレクションが存在しません")]
    CollectionIsNotFound,
    #[error("このコレクションは削除できません")]
    CollectionNotPermittedToDelete,
    #[error("コレクションはすでに存在しています")]
    CollectionIsAlreadyExist,
}
