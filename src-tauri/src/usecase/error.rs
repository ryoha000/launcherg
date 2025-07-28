use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("コレクションエレメントが存在しません")]
    CollectionElementIsNotFound,
}
