use thiserror::Error;

#[derive(Error, Debug)]
pub enum UseCaseError {
    #[error("Native Messaging Hostプロセスエラー: {0}")]
    NativeHostProcessError(String),
}
