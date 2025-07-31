use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("コレクションエレメントが存在しません")]
    CollectionElementIsNotFound,
    #[error("Native Messaging Hostプロセスエラー: {0}")]
    NativeHostProcessError(String),
}
